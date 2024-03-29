// SPDX-FileCopyrightText: 2020-2021 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Low-level PDF rendering utilities.
//!
//! This module provides low-level abstractions over [`printpdf`][]:  A [`Renderer`][] creates a
//! document with one or more pages with different sizes.  A [`Page`][] has one or more layers, all
//! of the same size.  A [`Layer`][] can be used to access its [`Area`][].
//!
//! An [`Area`][] is a view on a full layer or on a part of a layer.  It can be used to print
//! lines and text.  For more advanced text formatting, you can create a [`TextSection`][] from an
//! [`Area`][].
//!
//! [`printpdf`]: https://docs.rs/printpdf/latest/printpdf
//! [`Renderer`]: struct.Renderer.html
//! [`Page`]: struct.Page.html
//! [`Layer`]: struct.Layer.html
//! [`Area`]: struct.Area.html
//! [`TextSection`]: struct.TextSection.html

use std::cell;
use std::convert::TryInto;
use std::io;
use std::ops;
use std::rc;

use printpdf::ColorSpace;
use printpdf::ImageXObject;

use crate::elements::ColumnWidths;
use crate::error::{Context as _, Error, ErrorKind};
use crate::fonts;
use crate::style::{Color, LineStyle, Style};
use crate::utils::log_msg;
use crate::{Margins, Mm, Position, Size};

#[cfg(feature = "images")]
use crate::{Rotation, Scale};

/// A position relative to the top left corner of a layer.
struct LayerPosition(Position);

impl LayerPosition {
    pub fn from_area(area: &Area<'_>, position: Position) -> Self {
        Self(position + area.origin)
    }
}

/// A position relative to the bottom left corner of a layer (“user space” in PDF terms).
struct UserSpacePosition(Position);

impl UserSpacePosition {
    pub fn from_layer(layer: &Layer<'_>, position: LayerPosition) -> Self {
        Self(Position::new(
            position.0.x,
            layer.page.size.height - position.0.y,
        ))
    }
}

impl From<UserSpacePosition> for printpdf::Point {
    fn from(pos: UserSpacePosition) -> printpdf::Point {
        printpdf::Point::new(pos.0.x.into(), pos.0.y.into())
    }
}

impl ops::Deref for UserSpacePosition {
    type Target = Position;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Renders a PDF document with one or more pages.
///
/// This is a wrapper around a [`printpdf::PdfDocumentReference`][].
///
/// [`printpdf::PdfDocumentReference`]: https://docs.rs/printpdf/0.3.2/printpdf/types/pdf_document/struct.PdfDocumentReference.html
pub struct Renderer {
    doc: printpdf::PdfDocumentReference,
    // invariant: pages.len() >= 1
    pages: Vec<Page>,
}

impl Renderer {
    /// Creates a new PDF document renderer with one page of the given size and the given title.
    pub fn new(size: impl Into<Size>, title: impl AsRef<str>) -> Result<Renderer, Error> {
        let size = size.into();
        let (doc, page_idx, layer_idx) = printpdf::PdfDocument::new(
            title.as_ref(),
            size.width.into(),
            size.height.into(),
            "Layer 1",
        );
        let page_ref = doc.get_page(page_idx);
        let layer_ref = page_ref.get_layer(layer_idx);
        let page = Page::new(page_ref, layer_ref, size);

        Ok(Renderer {
            doc,
            pages: vec![page],
        })
    }

    /// Sets the PDF conformance for the generated PDF document.
    pub fn with_conformance(mut self, conformance: printpdf::PdfConformance) -> Self {
        self.doc = self.doc.with_conformance(conformance);
        self
    }

    /// Sets the creation date for the generated PDF document.
    pub fn with_creation_date(mut self, date: printpdf::OffsetDateTime) -> Self {
        self.doc = self.doc.with_creation_date(date);
        self
    }

    /// Sets the modification date for the generated PDF document.
    pub fn with_modification_date(mut self, date: printpdf::OffsetDateTime) -> Self {
        self.doc = self.doc.with_mod_date(date);
        self
    }

    /// Adds a new page with the given size to the document.
    pub fn add_page(&mut self, size: impl Into<Size>) {
        let size = size.into();
        let (page_idx, layer_idx) =
            self.doc
                .add_page(size.width.into(), size.height.into(), "Layer 1");
        let page_ref = self.doc.get_page(page_idx);
        let layer_ref = page_ref.get_layer(layer_idx);
        self.pages.push(Page::new(page_ref, layer_ref, size))
    }

    /// Returns the number of pages in this document.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Returns a page of this document.
    pub fn get_page(&self, idx: usize) -> Option<&Page> {
        self.pages.get(idx)
    }

    /// Returns a mutable reference to a page of this document.
    pub fn get_page_mut(&mut self, idx: usize) -> Option<&mut Page> {
        self.pages.get_mut(idx)
    }

    /// Returns a mutable reference to the first page of this document.
    pub fn first_page(&self) -> &Page {
        &self.pages[0]
    }

    /// Returns the first page of this document.
    pub fn first_page_mut(&mut self) -> &mut Page {
        &mut self.pages[0]
    }

    /// Returns the last page of this document.
    pub fn last_page(&self) -> &Page {
        &self.pages[self.pages.len() - 1]
    }

    /// Returns a mutable reference to the last page of this document.
    pub fn last_page_mut(&mut self) -> &mut Page {
        let idx = self.pages.len() - 1;
        &mut self.pages[idx]
    }

    /// Loads the font from the given data, adds it to the generated document and returns a
    /// reference to it.
    pub fn add_builtin_font(
        &self,
        builtin: printpdf::BuiltinFont,
    ) -> Result<printpdf::IndirectFontRef, Error> {
        match self.doc.add_builtin_font(builtin) {
            Ok(font) => Ok(font),
            Err(e) => Err(Error::new(
                format!("Failed to load font {}", e),
                ErrorKind::InvalidFont,
            )),
        }
    }

    /// Loads the font from the given data, adds it to the generated document and returns a
    /// reference to it.
    pub fn add_embedded_font(&self, data: &[u8]) -> Result<printpdf::IndirectFontRef, Error> {
        match self.doc.add_external_font(data) {
            Ok(font) => Ok(font),
            Err(e) => Err(Error::new(
                format!("Failed to load font {}", e),
                ErrorKind::InvalidFont,
            )),
        }
    }

    /// Writes this PDF document to a writer.
    pub fn write(self, w: impl io::Write) -> Result<(), Error> {
        self.doc
            .save(&mut io::BufWriter::new(w))
            .context("Failed to save document")
    }
}

/// A page of a PDF document.
///
/// This is a wrapper around a [`printpdf::PdfPageReference`][].
///
/// [`printpdf::PdfPageReference`]: https://docs.rs/printpdf/0.3.2/printpdf/types/pdf_page/struct.PdfPageReference.html
pub struct Page {
    page: printpdf::PdfPageReference,
    size: Size,
    layers: Layers,
}

impl Page {
    fn new(
        page: printpdf::PdfPageReference,
        layer: printpdf::PdfLayerReference,
        size: Size,
    ) -> Page {
        Page {
            page,
            size,
            layers: Layers::new(layer),
        }
    }

    /// Adds a new layer with the given name to the page.
    pub fn add_layer(&mut self, name: impl Into<String>) {
        let layer = self.page.add_layer(name);
        self.layers.push(layer);
    }

    /// Returns the number of layers on this page.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Returns a layer of this page.
    pub fn get_layer(&self, idx: usize) -> Option<Layer<'_>> {
        self.layers.get(idx).map(|l| Layer::new(self, l))
    }

    /// Returns the first layer of this page.
    pub fn first_layer(&self) -> Layer<'_> {
        Layer::new(self, self.layers.first())
    }

    /// Returns the last layer of this page.
    pub fn last_layer(&self) -> Layer<'_> {
        Layer::new(self, self.layers.last())
    }

    fn next_layer(&self, layer: &printpdf::PdfLayerReference) -> Layer<'_> {
        let layer = self.layers.next(layer).unwrap_or_else(|| {
            let layer = self
                .page
                .add_layer(format!("Layer {}", self.layers.len() + 1));
            self.layers.push(layer)
        });
        Layer::new(self, layer)
    }
}

#[derive(Debug)]
struct Layers(cell::RefCell<Vec<rc::Rc<LayerData>>>);

impl Layers {
    pub fn new(layer: printpdf::PdfLayerReference) -> Self {
        Self(vec![LayerData::from(layer).into()].into())
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn first(&self) -> rc::Rc<LayerData> {
        self.0.borrow().first().unwrap().clone()
    }

    pub fn last(&self) -> rc::Rc<LayerData> {
        self.0.borrow().last().unwrap().clone()
    }

    pub fn get(&self, idx: usize) -> Option<rc::Rc<LayerData>> {
        self.0.borrow().get(idx).cloned()
    }

    pub fn push(&self, layer: printpdf::PdfLayerReference) -> rc::Rc<LayerData> {
        let layer_data = rc::Rc::from(LayerData::from(layer));
        self.0.borrow_mut().push(layer_data.clone());
        layer_data
    }

    pub fn next(&self, layer: &printpdf::PdfLayerReference) -> Option<rc::Rc<LayerData>> {
        self.0
            .borrow()
            .iter()
            .skip_while(|l| l.layer.layer != layer.layer)
            .nth(1)
            .cloned()
    }
}

/// A layer of a page of a PDF document.
///
/// This is a wrapper around a [`printpdf::PdfLayerReference`][].
///
/// [`printpdf::PdfLayerReference`]: https://docs.rs/printpdf/0.3.2/printpdf/types/pdf_layer/struct.PdfLayerReference.html
#[derive(Clone)]
pub struct Layer<'p> {
    page: &'p Page,
    data: rc::Rc<LayerData>,
}

impl<'p> Layer<'p> {
    fn new(page: &'p Page, data: rc::Rc<LayerData>) -> Layer<'p> {
        Layer { page, data }
    }

    /// Returns the next layer of this page.
    ///
    /// If this layer is not the last layer, the existing next layer is used.  If it is the last
    /// layer, a new layer is created and added to the page.
    pub fn next(&self) -> Layer<'p> {
        self.page.next_layer(&self.data.layer)
    }

    /// Returns a drawable area for this layer.
    pub fn area(&self) -> Area<'p> {
        Area::new(self.clone(), Position::default(), self.page.size)
    }

    /// remove alpha channel from image x object
    pub fn remove_alpha_channel_from_image_x_object(image_x_object: ImageXObject) -> ImageXObject {
        if !matches!(image_x_object.color_space, ColorSpace::Rgba) {
            log_msg("Color space is not RGBA, skipping alpha channel removal.");
            return image_x_object;
        };
        log_msg("Color space is RGBA, removing alpha channel.");
        let ImageXObject {
            color_space,
            image_data,
            ..
        } = image_x_object;

        let new_image_data = image_data
            .chunks(4)
            .map(|rgba| {
                let [red, green, blue, alpha]: [u8; 4] = rgba.try_into().ok().unwrap();
                let alpha = alpha as f64 / 255.0;
                let new_red = ((1.0 - alpha) * 255.0 + alpha * red as f64) as u8;
                let new_green = ((1.0 - alpha) * 255.0 + alpha * green as f64) as u8;
                let new_blue = ((1.0 - alpha) * 255.0 + alpha * blue as f64) as u8;
                return [new_red, new_green, new_blue];
            })
            .collect::<Vec<[u8; 3]>>()
            .concat();

        let new_color_space = match color_space {
            ColorSpace::Rgba => ColorSpace::Rgb,
            ColorSpace::GreyscaleAlpha => ColorSpace::Greyscale,
            other_type => other_type,
        };

        ImageXObject {
            color_space: new_color_space,
            image_data: new_image_data,
            ..image_x_object
        }
    }

    #[cfg(feature = "images")]
    fn add_image(
        &self,
        image: &image::DynamicImage,
        position: LayerPosition,
        scale: Scale,
        rotation: Rotation,
        dpi: Option<f64>,
    ) {
        let has_alpha = image.color().has_alpha();
        let mut dynamic_image = printpdf::Image::from_dynamic_image(image);
        if has_alpha {
            // turn rbga to rgb
            dynamic_image.image =
                Self::remove_alpha_channel_from_image_x_object(dynamic_image.image);
        }
        let position = self.transform_position(position);
        dynamic_image.add_to_layer(
            self.data.layer.clone(),
            Some(position.x.into()),
            Some(position.y.into()),
            rotation.into(),
            Some(scale.x),
            Some(scale.y),
            dpi,
        );
    }

    fn add_line_shape<I>(&self, points: I)
    where
        I: IntoIterator<Item = LayerPosition>,
    {
        let line_points: Vec<_> = points
            .into_iter()
            .map(|pos| (self.transform_position(pos).into(), false))
            .collect();
        // log("add_line_shape", &format!("{:?}", line_points));
        let line = printpdf::Line {
            points: line_points,
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        self.data.layer.add_shape(line);
    }

    fn draw_filled_shape<I>(&self, points: I, color: Option<Color>)
    where
        I: IntoIterator<Item = LayerPosition>,
    {
        self.set_fill_color(color.clone());
        // fill color and outline color are the same
        if let Some(c) = color {
            self.set_outline_color(c);
        }
        let line_points: Vec<_> = points
            .into_iter()
            .map(|pos| (self.transform_position(pos).into(), false))
            .collect();
        // println!("filled shape line_points: {:?}", line_points);
        let line = printpdf::Line {
            points: line_points,
            is_closed: true,
            has_fill: true,
            has_stroke: true,
            is_clipping_path: false,
        };
        self.data.layer.add_shape(line);
    }

    fn set_fill_color(&self, color: Option<Color>) {
        if self.data.update_fill_color(color) {
            self.data
                .layer
                .set_fill_color(color.unwrap_or(Color::Rgb(0, 0, 0)).into());
        }
    }

    fn set_outline_thickness(&self, thickness: Mm) {
        if self.data.update_outline_thickness(thickness) {
            self.data
                .layer
                .set_outline_thickness(printpdf::Pt::from(thickness).0);
        }
    }

    fn set_outline_color(&self, color: Color) {
        if self.data.update_outline_color(color) {
            self.data.layer.set_outline_color(color.into());
        }
    }

    fn set_text_cursor(&self, cursor: LayerPosition) {
        let cursor = self.transform_position(cursor);
        self.data
            .layer
            .set_text_cursor(cursor.x.into(), cursor.y.into());
    }

    fn begin_text_section(&self) {
        self.data.layer.begin_text_section();
    }

    fn end_text_section(&self) {
        self.data.layer.end_text_section();
    }

    fn add_line_break(&self) {
        self.data.layer.add_line_break();
    }

    fn set_line_height(&self, line_height: Mm) {
        self.data.layer.set_line_height(line_height.0);
    }

    fn set_font(&self, font: &printpdf::IndirectFontRef, font_size: u8) {
        self.data.layer.set_font(font, font_size.into());
    }

    fn write_positioned_codepoints<P, C>(&self, positions: P, codepoints: C)
    where
        P: IntoIterator<Item = i64>,
        C: IntoIterator<Item = u16>,
    {
        self.data
            .layer
            .write_positioned_codepoints(positions.into_iter().zip(codepoints.into_iter()));
    }

    /// Transforms the given position that is relative to the upper left corner of the layer to a
    /// position that is relative to the lower left corner of the layer (as used by `printpdf`).
    fn transform_position(&self, position: LayerPosition) -> UserSpacePosition {
        UserSpacePosition::from_layer(self, position)
    }
}

#[derive(Debug)]
struct LayerData {
    layer: printpdf::PdfLayerReference,
    fill_color: cell::Cell<Color>,
    outline_color: cell::Cell<Color>,
    outline_thickness: cell::Cell<Mm>,
}

impl LayerData {
    pub fn update_fill_color(&self, color: Option<Color>) -> bool {
        let color = color.unwrap_or(Color::Rgb(0, 0, 0));
        self.fill_color.replace(color) != color
    }

    pub fn update_outline_color(&self, color: Color) -> bool {
        self.outline_color.replace(color) != color
    }

    pub fn update_outline_thickness(&self, thickness: Mm) -> bool {
        self.outline_thickness.replace(thickness) != thickness
    }
}

impl From<printpdf::PdfLayerReference> for LayerData {
    fn from(layer: printpdf::PdfLayerReference) -> Self {
        Self {
            layer,
            fill_color: Color::Rgb(0, 0, 0).into(),
            outline_color: Color::Rgb(0, 0, 0).into(),
            outline_thickness: Mm::from(printpdf::Pt(1.0)).into(),
        }
    }
}

/// A view on an area of a PDF layer that can be drawn on.
///
/// This struct provides access to the drawing methods of a [`printpdf::PdfLayerReference`][].  It
/// is defined by the layer that is drawn on and the origin and the size of the area.
///
/// [`printpdf::PdfLayerReference`]: https://docs.rs/printpdf/0.3.2/printpdf/types/pdf_layer/struct.PdfLayerReference.html
#[derive(Clone)]
pub struct Area<'p> {
    layer: Layer<'p>,
    origin: Position,
    size: Size,
    margin_top: Mm,
}

impl<'p> Area<'p> {
    fn new(layer: Layer<'p>, origin: Position, size: Size) -> Area<'p> {
        // println!("new area: y {:?}", origin.y);
        Area {
            layer,
            origin,
            size,
            margin_top: Mm(0.0),
        }
    }

    /// Returns a copy of this area on the next layer of the page.
    ///
    /// If this area is not on the last layer, the existing next layer is used.  If it is on the
    /// last layer, a new layer is created and added to the page.
    pub fn next_layer(&self) -> Self {
        let layer = self.layer.next();
        Self {
            layer,
            origin: self.origin,
            size: self.size,
            margin_top: self.margin_top,
        }
    }

    /// Reduces the size of the drawable area by the given margins.
    pub fn add_margins(&mut self, margins: impl Into<Margins>) {
        let margins = margins.into();
        self.origin.x += margins.left;
        self.origin.y += margins.top;
        self.size.width -= margins.left + margins.right;
        self.size.height -= margins.top + margins.bottom;
        self.margin_top += margins.top;
    }

    /// Returns the size of this area.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Adds the given offset to the area, reducing the drawable area.
    pub fn add_offset(&mut self, offset: impl Into<Position>) {
        let offset = offset.into();
        self.origin.x += offset.x;
        self.origin.y += offset.y;
        self.size.width -= offset.x;
        self.size.height -= offset.y;
    }

    /// add left x
    ///
    pub fn add_left(&mut self, left: Mm) {
        self.origin.x += left;
    }

    /// get start x
    pub fn start_x(&self) -> Mm {
        self.origin.x
    }

    /// get start y
    pub fn start_y(&self) -> Mm {
        self.origin.y
    }

    /// get margin_top
    pub fn get_margin_top(&self) -> Mm {
        self.margin_top
    }

    /// Sets the size of this area.
    pub fn set_size(&mut self, size: impl Into<Size>) {
        self.size = size.into();
    }

    /// Sets the width of this area.
    pub fn set_width(&mut self, width: Mm) {
        self.size.width = width;
    }

    /// Sets the height of this area.
    pub fn set_height(&mut self, height: Mm) {
        self.size.height = height;
    }

    /// Splits this area horizontally using the given weights/pixels.
    pub fn split_horizontally(&self, weights: &ColumnWidths) -> Vec<Area<'p>> {
        match weights {
            ColumnWidths::Weights(weights) => self.split_horizontally_by_weights(weights),
            ColumnWidths::PixelWidths(widths) => self.split_horizontally_by_pixels(widths),
        }
    }

    /// Splits this area horizontally using the given weights.
    ///
    /// The returned vector has the same number of elements as the provided slice.  The width of
    /// the *i*-th area is *width \* weights[i] / total_weight*, where *width* is the width of this
    /// area, and *total_weight* is the sum of all given weights.
    fn split_horizontally_by_weights(&self, weights: &[usize]) -> Vec<Area<'p>> {
        let total_weight: usize = weights.iter().sum();
        let factor = self.size.width / total_weight as f64;
        let widths = weights.iter().map(|weight| factor * *weight as f64);
        let mut offset = Mm(0.0);
        let mut areas = Vec::new();
        for width in widths {
            let mut area = self.clone();
            area.origin.x += offset;
            area.size.width = width;
            areas.push(area);
            offset += width;
        }
        areas
    }

    /// Splits this area horizontally using the given pixel weights.
    ///
    /// The returned vector has the same number of elements as the provided slice.  The width of
    /// the *i*-th area is *width \* weights[i] / total_weight*, where *width* is the width of this
    /// area, and *total_weight* is the sum of all given weights.
    fn split_horizontally_by_pixels(&self, widths: &[f64]) -> Vec<Area<'p>> {
        let mut offset = Mm(0.0);
        let mut areas = Vec::new();
        for width in widths {
            let mut area = self.clone();
            area.origin.x += offset;
            area.size.width = Mm::from(*width);
            areas.push(area);
            offset += Mm::from(*width);
        }
        areas
    }

    /// Inserts an image into the document.
    ///
    /// *Only available if the `images` feature is enabled.*
    ///
    /// The position is assumed to be relative to the upper left hand corner of the area.
    /// Your position will need to compensate for rotation/scale/dpi. Using [`Image`][]’s
    /// render functionality will do this for you and is the recommended way to
    /// insert an image into an Area.
    ///
    /// [`Image`]: ../elements/struct.Image.html
    #[cfg(feature = "images")]
    pub fn add_image(
        &self,
        image: &image::DynamicImage,
        position: Position,
        scale: Scale,
        rotation: Rotation,
        dpi: Option<f64>,
    ) {
        self.layer
            .add_image(image, self.position(position), scale, rotation, dpi);
    }

    /// Draws a line with the given points and the given line style.
    ///
    /// The points are relative to the upper left corner of the area.
    pub fn draw_line<I>(&self, points: I, line_style: LineStyle)
    where
        I: IntoIterator<Item = Position>,
    {
        self.layer.set_outline_thickness(line_style.thickness());
        self.layer.set_outline_color(line_style.color());
        self.layer
            .add_line_shape(points.into_iter().map(|pos| self.position(pos)));
    }

    /// Draws a line with the given points and the given line style.
    ///
    /// The points are relative to the upper left corner of the area.
    pub fn draw_filled_shape<I>(&self, points: I, color: Option<Color>, line_style: LineStyle)
    where
        I: IntoIterator<Item = Position>,
    {
        self.layer.set_outline_thickness(line_style.thickness());
        self.layer
            .draw_filled_shape(points.into_iter().map(|pos| self.position(pos)), color);
    }

    /// Tries to draw the given string at the given position and returns `true` if the area was
    /// large enough to draw the string.
    ///
    /// The font cache must contain the PDF font for the font set in the style.  The position is
    /// relative to the upper left corner of the area.
    pub fn print_str<S: AsRef<str>>(
        &self,
        font_cache: &fonts::FontCache,
        position: Position,
        style: Style,
        s: S,
    ) -> Result<bool, Error> {
        if let Some(mut section) =
            self.text_section(font_cache, position, style.metrics(font_cache))
        {
            section.print_str(s, style)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Creates a new text section at the given position if the text section fits in this area.
    ///
    /// The given style is only used to calculate the line height of the section.  The position is
    /// relative to the upper left corner of the area.  The font cache must contain the PDF font
    /// for all fonts printed with the text section.
    pub fn text_section<'f>(
        &self,
        font_cache: &'f fonts::FontCache,
        position: Position,
        metrics: fonts::Metrics,
    ) -> Option<TextSection<'f, 'p>> {
        let mut area = self.clone();
        area.add_offset(position);
        TextSection::new(font_cache, area, metrics)
    }

    /// Returns a position relative to the top left corner of this area.
    fn position(&self, position: Position) -> LayerPosition {
        LayerPosition::from_area(self, position)
    }
}

/// A text section that is drawn on an area of a PDF layer.
pub struct TextSection<'f, 'p> {
    font_cache: &'f fonts::FontCache,
    area: Area<'p>,
    is_first: bool,
    metrics: fonts::Metrics,
    font: Option<(printpdf::IndirectFontRef, u8)>,
}

impl<'f, 'p> TextSection<'f, 'p> {
    fn new(
        font_cache: &'f fonts::FontCache,
        area: Area<'p>,
        metrics: fonts::Metrics,
    ) -> Option<TextSection<'f, 'p>> {
        if metrics.glyph_height > area.size.height {
            return None;
        }

        area.layer.begin_text_section();
        area.layer.set_line_height(metrics.line_height);

        Some(TextSection {
            font_cache,
            area,
            is_first: true,
            metrics,
            font: None,
        })
    }

    fn set_text_cursor(&self, x_offset: Mm) {
        let cursor = self
            .area
            .position(Position::new(x_offset, self.metrics.ascent));
        self.area.layer.set_text_cursor(cursor);
    }

    fn set_font(&mut self, font: &printpdf::IndirectFontRef, font_size: u8) {
        let font_is_set = self
            .font
            .as_ref()
            .map(|(font, font_size)| (font, *font_size))
            .map(|data| data == (font, font_size))
            .unwrap_or_default();
        if !font_is_set {
            self.font = Some((font.clone(), font_size));
            self.area.layer.set_font(font, font_size);
        }
    }

    /// Tries to add a new line and returns `true` if the area was large enough to fit the new
    /// line.
    #[must_use]
    pub fn add_newline(&mut self) -> bool {
        if self.metrics.line_height > self.area.size.height {
            false
        } else {
            self.area.layer.add_line_break();
            self.area.add_offset((0, self.metrics.line_height));
            true
        }
    }

    /// Prints the given string with the given style.
    ///
    /// The font cache for this text section must contain the PDF font for the given style.
    pub fn print_str(&mut self, s: impl AsRef<str>, style: Style) -> Result<(), Error> {
        let s = s.as_ref();
        let font = style.font(self.font_cache);
        // Adjust cursor to remove left bearing of the first character of the first string
        if self.is_first {
            let x_offset = if let Some(first_c) = s.chars().next() {
                style.char_left_side_bearing(self.font_cache, first_c) * -1.0
            } else {
                Mm(0.0)
            };
            self.set_text_cursor(x_offset);
        }
        self.is_first = false;

        let positions = font
            .kerning(self.font_cache, s.chars())
            .into_iter()
            // Kerning is measured in 1/1000 em
            .map(|pos| pos * -1000.0)
            .map(|pos| pos as i64);
        let codepoints = if font.is_builtin() {
            // Built-in fonts always use the Windows-1252 encoding
            encode_win1252(s)?
        } else {
            font.glyph_ids(&self.font_cache, s.chars())
        };

        let font = self
            .font_cache
            .get_pdf_font(font)
            .expect("Could not find PDF font in font cache");
        self.area.layer.set_fill_color(style.color());
        self.set_font(font, style.font_size());

        // println!("codepoints: {:?}", codepoints);

        self.area
            .layer
            .write_positioned_codepoints(positions, codepoints);
        Ok(())
    }
}

impl<'f, 'p> Drop for TextSection<'f, 'p> {
    fn drop(&mut self) {
        self.area.layer.end_text_section();
    }
}

/// Encodes the given string using the Windows-1252 encoding for use with built-in PDF fonts,
/// returning an error if it contains unsupported characters.
fn encode_win1252(s: &str) -> Result<Vec<u16>, Error> {
    let bytes: Vec<_> = lopdf::Document::encode_text(Some("WinAnsiEncoding"), s)
        .into_iter()
        .map(u16::from)
        .collect();

    // Windows-1252 is a single-byte encoding, so one byte is one character.
    if bytes.len() != s.chars().count() {
        Err(Error::new(
            format!(
                "Tried to print a string with characters that are not supported by the \
                Windows-1252 encoding with a built-in font: {}",
                s
            ),
            ErrorKind::UnsupportedEncoding,
        ))
    } else {
        Ok(bytes)
    }
}
