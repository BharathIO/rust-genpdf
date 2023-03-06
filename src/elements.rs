// SPDX-FileCopyrightText: 2020-2021 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Elements of a PDF document.
//!
//! This module provides implementations of the [`Element`][] trait that can be used to render and
//! arrange text and shapes.
//!
//! It includes the following elements:
//! - Containers:
//!   - [`LinearLayout`][]: arranges its elements sequentially
//!   - [`TableLayout`][]: arranges its elements in columns and rows
//!   - [`OrderedList`][] and [`UnorderedList`][]: arrange their elements sequentially with bullet
//!     points
//! - Text:
//!   - [`Text`][]: a single line of text
//!   - [`Paragraph`][]: a wrapped and aligned paragraph of text
//! - Wrappers:
//!   - [`FramedElement`][]: draws a frame around the wrapped element
//!   - [`PaddedElement`][]: adds a padding to the wrapped element
//!   - [`StyledElement`][]: sets a default style for the wrapped element and its children
//! - Other:
//!   - [`Image`][]: an image (requires the `images` feature)
//!   - [`Break`][]: adds forced line breaks as a spacer
//!   - [`PageBreak`][]: adds a forced page break
//!
//! You can create custom elements by implementing the [`Element`][] trait.
//!
//! [`Element`]: ../trait.Element.html
//! [`LinearLayout`]: struct.LinearLayout.html
//! [`TableLayout`]: struct.TableLayout.html
//! [`OrderedList`]: struct.OrderedList.html
//! [`UnorderedList`]: struct.UnorderedList.html
//! [`Text`]: struct.Text.html
//! [`Image`]: struct.Image.html
//! [`Break`]: struct.Break.html
//! [`PageBreak`]: struct.PageBreak.html
//! [`Paragraph`]: struct.Paragraph.html
//! [`FramedElement`]: struct.FramedElement.html
//! [`PaddedElement`]: struct.PaddedElement.html
//! [`StyledElement`]: struct.StyledElement.html

#[cfg(feature = "images")]
mod images;

use std::collections;
use std::iter;
use std::mem;

use crate::error::{Error, ErrorKind};
use crate::fonts;
use crate::render;
use crate::style;
use crate::style::Color;
use crate::style::{LineStyle, Style, StyledString};
use crate::utils::log;
use crate::wrap;
use crate::{Alignment, Context, Element, Margins, Mm, Position, RenderResult, Size};

#[cfg(feature = "images")]
pub use images::Image;

/// Helper trait for creating boxed elements.
pub trait IntoBoxedElement {
    /// Creates a boxed element from this element.
    fn into_boxed_element(self) -> Box<dyn Element>;
}

impl<E: Element + 'static> IntoBoxedElement for E {
    fn into_boxed_element(self) -> Box<dyn Element> {
        Box::new(self)
    }
}

impl IntoBoxedElement for Box<dyn Element> {
    fn into_boxed_element(self) -> Box<dyn Element> {
        self
    }
}

/// Arranges a list of elements sequentially.
///
/// Currently, elements can only be arranged vertically.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut layout = elements::LinearLayout::vertical();
/// layout.push(elements::Paragraph::new("Test1"));
/// layout.push(elements::Paragraph::new("Test2"));
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let layout = elements::LinearLayout::vertical()
///     .element(elements::Paragraph::new("Test1"))
///     .element(elements::Paragraph::new("Test2"));
/// ```
///
pub struct LinearLayout {
    elements: Vec<Box<dyn Element>>,
    render_idx: usize,
    margins: Option<Margins>,
    list_item_spacing: f64,
}

impl LinearLayout {
    fn new() -> LinearLayout {
        LinearLayout {
            elements: Vec::new(),
            render_idx: 0,
            margins: None,
            list_item_spacing: 0.0,
        }
    }

    /// Creates a new linear layout that arranges its elements vertically.
    pub fn vertical() -> LinearLayout {
        LinearLayout::new()
    }

    /// set margins
    /// margins is the distance between the text and the border
    pub fn set_margins(&mut self, margins: Margins) {
        self.margins = Some(margins);
    }

    /// returns the current margins
    pub fn get_margins(&self) -> Option<Margins> {
        self.margins
    }

    /// set list item margins
    pub fn set_list_item_spacing(&mut self, spacing: f64) {
        self.list_item_spacing = spacing;
    }

    /// Adds the given element to this layout.
    pub fn push<E: IntoBoxedElement>(&mut self, element: E) {
        self.elements.push(element.into_boxed_element());
    }

    /// Adds the given element to this layout and it returns the layout.
    pub fn element<E: IntoBoxedElement>(mut self, element: E) -> Self {
        self.push(element);
        self
    }

    fn render_vertical(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        if let Some(margins) = self.margins {
            area.add_margins(margins);
        }
        while area.size().height > Mm(0.0) && self.render_idx < self.elements.len() {
            let element_result =
                self.elements[self.render_idx].render(context, area.clone(), style)?;
            let mut left_offset = 0;
            let right_offset = element_result.size.height + Mm(self.list_item_spacing);
            if let Some(el_offset) = element_result.offset {
                left_offset = el_offset.0 as i32;
            }
            area.add_offset(Position::new(left_offset, right_offset));
            result.size = result.size.stack_vertical(element_result.size);
            result.size.height += Mm(self.list_item_spacing);
            if element_result.has_more {
                result.has_more = true;
                return Ok(result);
            }
            self.render_idx += 1;
        }
        result.has_more = self.render_idx < self.elements.len();
        if let Some(margins) = self.margins {
            result.size.height += margins.top + margins.bottom;
        }
        Ok(result)
    }
}

impl Element for LinearLayout {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        // TODO: add horizontal layout
        self.render_vertical(context, area, style)
    }

    fn get_probable_height(
        &mut self,
        style: Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        let mut h = self
            .elements
            .iter_mut()
            .map(|e| e.get_probable_height(style, context, area.clone()))
            .sum();
        if let Some(margins) = self.margins {
            h += margins.top + margins.bottom;
        }
        h
    }
}

impl<E: IntoBoxedElement> iter::Extend<E> for LinearLayout {
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        self.elements
            .extend(iter.into_iter().map(|e| e.into_boxed_element()))
    }
}

/// A single line of formatted text.
///
/// This element renders a single styled string on a single line.  It does not wrap it if the
/// string is longer than the line.  Therefore you should prefer [`Paragraph`][] over `Text` for
/// most use cases.
///
/// [`Paragraph`]: struct.Paragraph.html
#[derive(Clone, Debug, Default)]
pub struct Text {
    text: StyledString,
}

impl Text {
    /// Creates a new instance with the given styled string.
    pub fn new(text: impl Into<StyledString>) -> Text {
        Text { text: text.into() }
    }
}

impl Element for Text {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        mut style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        style.merge(self.text.style);
        if area.print_str(
            &context.font_cache,
            Position::default(),
            style,
            &self.text.s,
        )? {
            result.size = Size::new(
                style.str_width(&context.font_cache, &self.text.s),
                style.line_height(&context.font_cache),
            );
        } else {
            result.has_more = true;
        }
        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        _area: render::Area<'_>,
    ) -> Mm {
        style.line_height(&context.font_cache)
    }
}

/// A multi-line wrapped paragraph of formatted text.
///
/// If the text of this paragraph is longer than the page width, the paragraph is wrapped at word
/// borders (and additionally at string borders if it contains multiple strings).  If a word in the
/// paragraph is longer than the page width, the text is truncated.
///
/// Use the [`push`][], [`string`][], [`push_styled`][] and [`string_styled`][] methods to add
/// strings to this paragraph.  Besides the styling of the text (see [`Style`][]), you can also set
/// an [`Alignment`][] for the paragraph.
///
/// The line height and spacing are calculated based on the style of each string.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::{elements, style};
/// let mut p = elements::Paragraph::default();
/// p.push("This is an ");
/// p.push_styled("important", style::Color::Rgb(255, 0, 0));
/// p.push(" message!");
/// p.set_alignment(genpdf::Alignment::Center);
/// ```
///
/// Chained:
/// ```
/// use genpdf::{elements, style};
/// let p = elements::Paragraph::default()
///     .string("This is an ")
///     .styled_string("important", style::Color::Rgb(255, 0, 0))
///     .string(" message!")
///     .aligned(genpdf::Alignment::Center);
/// ```
///
/// [`Style`]: ../style/struct.Style.html
/// [`Alignment`]: ../enum.Alignment.html
/// [`Element::styled`]: ../trait.Element.html#method.styled
/// [`push`]: #method.push
/// [`push_styled`]: #method.push_styled
/// [`string`]: #method.string
/// [`string_styled`]: #method.string_styled
#[derive(Clone, Debug, Default)]
pub struct Paragraph {
    text: Vec<StyledString>,
    words: collections::VecDeque<StyledString>,
    style_applied: bool,
    alignment: Alignment,
    style: style::Style,
    margins: Option<Margins>,
}

impl Paragraph {
    /// Creates a new paragraph with the given content.
    pub fn new(text: impl Into<StyledString>) -> Paragraph {
        Paragraph {
            text: vec![text.into()],
            style: style::Style::new(),
            ..Default::default()
        }
    }

    /// set font size
    pub fn set_font_size(&mut self, size: u8) {
        self.style.set_font_size(size);
    }

    /// Sets the line spacing factor for this style.
    pub fn set_line_spacing(&mut self, line_spacing: f64) {
        self.style.set_line_spacing(line_spacing);
    }

    /// Set color
    pub fn set_color(&mut self, color: style::Color) {
        self.style.set_color(color);
    }

    /// set font bold
    pub fn set_bold(&mut self, bold: bool) {
        self.style.set_bold(bold);
    }

    /// Sets the underline effect for this style.
    pub fn set_underline(&mut self, underline: bool) {
        self.style.set_underline(underline);
    }

    /// Returns whether the underline text effect is set.
    pub fn is_underline(&self) -> bool {
        self.style.is_underline()
    }

    /// set font italic
    pub fn set_italic(&mut self, italic: bool) {
        self.style.set_italic(italic);
    }

    /// set margins
    /// margins is the distance between the text and the border
    pub fn set_margins(&mut self, margins: Margins) {
        self.margins = Some(margins);
    }

    /// returns the current padding
    pub fn get_margins(&self) -> Option<Margins> {
        self.margins
    }

    /// Sets the alignment of this paragraph.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }

    /// Sets the alignment of this paragraph and returns the paragraph.
    pub fn aligned(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }

    /// Adds a string to the end of this paragraph.
    pub fn push(&mut self, s: impl Into<StyledString>) {
        self.text.push(s.into());
    }

    /// Adds a string to the end of this paragraph and returns the paragraph.
    pub fn string(mut self, s: impl Into<StyledString>) -> Self {
        self.push(s);
        self
    }

    /// Adds a string with the given style to the end of this paragraph.
    pub fn push_styled(&mut self, s: impl Into<String>, style: impl Into<Style>) {
        self.text.push(StyledString::new(s, style))
    }

    /// Adds a string with the given style to the end of this paragraph and returns the paragraph.
    pub fn styled_string(mut self, s: impl Into<String>, style: impl Into<Style>) -> Self {
        self.push_styled(s, style);
        self
    }

    fn get_offset(&self, width: Mm, max_width: Mm) -> Mm {
        match self.alignment {
            Alignment::Left => Mm::default(),
            Alignment::Center => (max_width - width) / 2.0,
            Alignment::Right => max_width - width,
        }
    }

    fn apply_style(&mut self, doc_style: Style) {
        if !self.style_applied {
            for s in &mut self.text {
                // s.style = style.and(s.style);
                // s.style = style.and(s.style);
                // s.style = s.style.and(style);
                // s.style = s.style.and(self.style);
                // println!("s.style {:?}", s.style);
                let para_style = self.style;
                let str_style = s.style;
                let source_style = doc_style.and(para_style);
                // println!("Before s {:?}, cs {:?}", s, source_style);
                s.style = source_style.and(str_style);
                // println!("After s {:?}, s.style {:?}", s, s.style);
                // s.style = cs.override_with(s.style);
            }
            self.style_applied = true;
        }
    }
}

fn replace_page_number(
    words: collections::VecDeque<StyledString>,
    context: &Context,
) -> collections::VecDeque<StyledString> {
    let mut words_copy = words.clone();
    // loop words and replace #{page} with context.page_number & remove new lines
    for i in 0..words.len() {
        let mut s = words[i].s.clone();
        s = s.replace("\n", "");
        if s.contains(&"#{page}") {
            let page = context.page_number;
            s = s.replace(&"#{page}", &page.to_string());
        }
        words_copy[i].s = s.into();
    }
    words_copy
}

impl Element for Paragraph {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        self.apply_style(style);

        if self.words.is_empty() {
            if self.text.is_empty() {
                return Ok(result);
            }
            self.words = wrap::Words::new(mem::take(&mut self.text)).collect();
            self.words = replace_page_number(self.words.clone(), context);
        }

        if let Some(margins) = self.margins {
            area.add_margins(margins);
        }

        let words = self.words.iter().map(Into::into);
        let mut rendered_len = 0;
        let mut wrapper = wrap::Wrapper::new(words, context, area.size().width);
        for (line, delta) in &mut wrapper {
            let width = line.iter().map(|s| s.width(&context.font_cache)).sum();
            // Calculate the maximum line height
            let metrics = line
                .iter()
                .map(|s| s.style.metrics(&context.font_cache))
                .fold(fonts::Metrics::default(), |max, m| max.max(&m));
            let height = metrics.line_height;
            let x = self.get_offset(width, area.size().width);
            let position = Position::new(x, 0);

            // println!("x {:?}", x);
            let mut line_width = Mm(0.0);
            if let Some(mut section) = area.text_section(&context.font_cache, position, metrics) {
                for s in line {
                    section.print_str(&s.s, s.style)?;
                    let s_width = s.width(&context.font_cache);
                    // println!("s {:?}, {:?}", s.s, s.style);
                    if s.style.is_underline() {
                        let ls = LineStyle::new().with_thickness(0.2);
                        let left = x + line_width;
                        let line_offset = ls.thickness() / 2.0;
                        let right = left + s_width;
                        let bottom = metrics.line_height;
                        let bottom_points = vec![
                            Position::new(left, bottom - line_offset),
                            Position::new(right, bottom - line_offset),
                        ];
                        area.draw_line(bottom_points, ls);
                    }
                    line_width += s_width;
                    rendered_len += s.s.len();
                }
                rendered_len -= delta;
            } else {
                result.has_more = true;
                break;
            }
            result.size = result
                .size
                .stack_vertical(Size::new(width, metrics.line_height));
            // println!("rendered_len: {:?}", rendered_len);
            // println!("result.size: {:?}", result.size);

            area.add_offset(Position::new(0, height));
        }

        if wrapper.has_overflowed() {
            // extract text from words
            let mut text = String::new();
            for s in &self.words {
                text.push_str(&s.s);
            }
            let msg = format!(
                "Page overflowed while trying to wrap a string \"{}\", please increase the component's width.",
                text
            );
            return Err(Error::new(msg, ErrorKind::PageSizeExceeded));
        }

        // Remove the rendered data from self.words so that we don’t render it again on the next
        // call to render.
        while rendered_len > 0 && !self.words.is_empty() {
            if self.words[0].s.len() <= rendered_len {
                rendered_len -= self.words[0].s.len();
                self.words.pop_front();
            } else {
                self.words[0].s.replace_range(..rendered_len, "");
                rendered_len = 0;
            }
        }

        if let Some(margins) = self.margins {
            result.size.width += margins.left + margins.right;
            result.size.height += margins.top + margins.bottom;
        }
        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        self.apply_style(style);
        let mut height = Mm::default();
        let mut words = wrap::Words::new(self.text.clone()).collect();
        words = replace_page_number(words, context);
        let mut wrapper =
            wrap::Wrapper::new(words.iter().map(Into::into), context, area.size().width);
        for (line, _) in &mut wrapper {
            let metrics = line
                .iter()
                .map(|s| s.style.metrics(&context.font_cache))
                .fold(fonts::Metrics::default(), |max, m| max.max(&m));
            height += metrics.line_height;
        }
        if let Some(margins) = self.margins {
            height += margins.top + margins.bottom;
        }
        height
    }
}

impl From<Vec<StyledString>> for Paragraph {
    fn from(text: Vec<StyledString>) -> Paragraph {
        Paragraph {
            text,
            ..Default::default()
        }
    }
}

impl<T: Into<StyledString>> iter::Extend<T> for Paragraph {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for s in iter {
            self.push(s);
        }
    }
}

impl<T: Into<StyledString>> iter::FromIterator<T> for Paragraph {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut paragraph = Paragraph::default();
        paragraph.extend(iter);
        paragraph
    }
}

/// A line break.
///
/// This element inserts a given number of empty lines.
///
/// # Example
///
/// ```
/// // Draws 5 empty lines (calculating the line height using the current style)
/// let b = genpdf::elements::Break::new(5);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Break {
    lines: f64,
}

impl Break {
    /// Creates a new break with the given number of lines.
    pub fn new(lines: impl Into<f64>) -> Break {
        Break {
            lines: lines.into(),
        }
    }
}

impl Element for Break {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        if self.lines <= 0.0 {
            return Ok(result);
        }
        let line_height = style.line_height(&context.font_cache);
        let break_height = line_height * self.lines;
        if break_height < area.size().height {
            result.size.height = break_height;
            self.lines = 0.0;
        } else {
            result.size.height = area.size().height;
            self.lines -= result.size.height.0 / line_height.0;
        }
        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        let line_height = style.line_height(&context.font_cache);
        let break_height = line_height * self.lines;
        if break_height < area.size().height {
            break_height
        } else {
            area.size().height
        }
    }
}

/// A page break.
///
/// This element inserts a page break.
///
/// # Example
///
/// ```
/// let pb = genpdf::elements::PageBreak::new();
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct PageBreak {
    cont: bool,
}

impl PageBreak {
    /// Creates a new page break.
    pub fn new() -> PageBreak {
        PageBreak::default()
    }
}

impl Element for PageBreak {
    fn render(
        &mut self,
        _context: &Context,
        _area: render::Area<'_>,
        _style: Style,
    ) -> Result<RenderResult, Error> {
        if self.cont {
            Ok(RenderResult::default())
        } else {
            // We don’t use (0,0) as the size as this might abort the render process if this is the
            // first element on a new page, see the Rendering Process section of the crate
            // documentation.
            self.cont = true;
            Ok(RenderResult {
                size: Size::new(1, 0),
                has_more: true,
                offset: None,
            })
        }
    }

    fn get_probable_height(
        &mut self,
        _style: style::Style,
        _context: &Context,
        _area: render::Area<'_>,
    ) -> Mm {
        Mm::default()
    }
}

/// A line.
///
/// This element inserts a line.
///
/// # Example
///
/// ```
// let line = genpdf::elements::Line::new();
/// ```
#[derive(Clone, Debug)]
pub struct Line {
    thickness: Mm,
    color: Color,
    width: Option<Mm>,  // width is only used for horizontal lines
    height: Option<Mm>, // height is only used for vertical lines
    orientation: String,
}

impl Default for Line {
    fn default() -> Line {
        Line {
            thickness: Mm::from(0.1),
            color: Color::Rgb(0, 0, 0),
            width: None,
            height: None,
            orientation: "horizontal".to_string(),
        }
    }
}

impl Line {
    /// Creates a new line.
    pub fn new() -> Line {
        Line::default()
    }

    /// Sets the thickness of the line.
    pub fn with_thickness(mut self, thickness: impl Into<Mm>) -> Line {
        self.thickness = thickness.into();
        self
    }

    /// Sets the color of the line.
    pub fn with_color(mut self, color: Color) -> Line {
        self.color = color;
        self
    }

    /// Sets the width of the line.
    pub fn with_width(mut self, width: impl Into<Mm>) -> Line {
        self.width = Some(width.into());
        self
    }

    /// Sets the height of the line.
    pub fn with_height(mut self, height: impl Into<Mm>) -> Line {
        self.height = Some(height.into());
        self
    }

    /// Sets the orientation of the line.
    pub fn with_orientation(mut self, orientation: impl Into<String>) -> Line {
        self.orientation = orientation.into();
        self
    }

    /// Returns the line thickness.
    pub fn thickness(&self) -> Mm {
        self.thickness
    }

    /// Returns the line color.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Returns the line width.
    pub fn width(&self) -> Option<Mm> {
        self.width
    }

    /// Returns the line orientation.
    pub fn orientation(&self) -> &str {
        self.orientation.as_str()
    }

    /// Returns the line height.
    pub fn height(&self) -> Option<Mm> {
        self.height
    }
}

impl Line {
    fn render_horizontal_line(
        &mut self,
        mut area: render::Area<'_>,
    ) -> Result<RenderResult, Error> {
        let top_thickness = self.thickness();
        let line_offset = top_thickness / 2.0;
        let area_width = area.size().width;
        let top = Mm::from(0.0);
        let left = Mm::from(0.0);
        let right = area_width;

        let line_start_x = left;
        let line_end_x = right;
        let line_start_y = top + line_offset; // top_thickness + line_offset
        let line_end_y = top + line_offset; // top_thickness + line_offset

        let top_points = vec![
            Position::new(line_start_x, line_start_y),
            Position::new(line_end_x, line_end_y),
        ];
        let top_line = LineStyle::default()
            .with_thickness(top_thickness)
            .with_color(self.color());
        area.draw_line(top_points, top_line);

        let mut result = RenderResult::default();
        result.size.height = top_thickness;
        area.add_offset(Position::new(0, result.size.height));
        Ok(result)
    }

    fn render_vertical_line(&mut self, area: render::Area<'_>) -> Result<RenderResult, Error> {
        let left_thickness = self.thickness();
        let line_offset = left_thickness / 2.0;
        let area_height = match self.height() {
            Some(height) => height,
            None => area.size().height,
        };

        let top = Mm::from(0.0);
        let left = Mm::from(0.0);
        let bottom = area_height;
        let line_start_x = left + line_offset;
        let line_end_x = left + line_offset;
        let line_start_y = top;
        let line_end_y = bottom;

        let left_points = vec![
            Position::new(line_start_x, line_start_y),
            Position::new(line_end_x, line_end_y),
        ];
        let left_line = LineStyle::default()
            .with_thickness(left_thickness)
            .with_color(self.color());
        // log("left_points", &format!("{:?}", left_points));
        area.draw_line(left_points, left_line);

        let mut render_result = RenderResult::default();
        // render_result.size.height = area_height - top_thickness;
        render_result.size.width = left_thickness;
        render_result.offset = Some(left_thickness);
        Ok(render_result)
    }
}

impl Element for Line {
    fn render(
        &mut self,
        _context: &Context,
        area: render::Area<'_>,
        _style: Style,
    ) -> Result<RenderResult, Error> {
        match self.orientation() {
            "vertical" => self.render_vertical_line(area),
            _ => self.render_horizontal_line(area),
        }
    }

    fn get_probable_height(
        &mut self,
        _style: style::Style,
        _context: &Context,
        _area: render::Area<'_>,
    ) -> Mm {
        match self.orientation() {
            "vertical" => self.height().unwrap_or(_area.size().height),
            _ => self.thickness(),
        }
    }
}

/// Adds a padding to the wrapped element.
///
/// # Examples
///
/// Direct usage:
/// ```
/// use genpdf::elements;
/// let p = elements::PaddedElement::new(
///     elements::Paragraph::new("text"),
///     genpdf::Margins::trbl(5, 2, 5, 10),
/// );
/// ```
///
/// Using [`Element::padded`][]:
/// ```
/// use genpdf::{elements, Element as _};
/// let p = elements::Paragraph::new("text")
///     .padded(genpdf::Margins::trbl(5, 2, 5, 10));
/// ```
///
/// [`Element::padded`]: ../trait.Element.html#method.padded
#[derive(Clone, Debug, Default)]
pub struct PaddedElement<E: Element> {
    element: E,
    padding: Margins,
}

impl<E: Element> PaddedElement<E> {
    /// Creates a new padded element that wraps the given element with the given padding.
    pub fn new(element: E, padding: impl Into<Margins>) -> PaddedElement<E> {
        PaddedElement {
            element,
            padding: padding.into(),
        }
    }
}

impl<E: Element> Element for PaddedElement<E> {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        area.add_margins(Margins {
            bottom: Mm(0.0),
            ..self.padding
        });
        let mut result = self.element.render(context, area, style)?;
        result.size.width += self.padding.left + self.padding.right;
        result.size.height += self.padding.top + self.padding.bottom;
        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        let mut area = area;
        area.add_margins(Margins {
            bottom: Mm(0.0),
            ..self.padding
        });
        self.element.get_probable_height(style, context, area)
            + self.padding.top
            + self.padding.bottom
    }
}

/// Adds a default style to the wrapped element and its children.
///
/// # Examples
///
/// Direct usage:
/// ```
/// use genpdf::{elements, style};
/// let p = elements::StyledElement::new(
///     elements::Paragraph::new("text"),
///     style::Effect::Bold,
/// );
/// ```
///
/// Using [`Element::styled`][]:
/// ```
/// use genpdf::{elements, style, Element as _};
/// let p = elements::Paragraph::new("text")
///     .styled(style::Effect::Bold);
/// ```
///
/// [`Element::styled`]: ../trait.Element.html#method.styled
#[derive(Clone, Debug, Default)]
pub struct StyledElement<E: Element> {
    element: E,
    style: Style,
}

impl<E: Element> StyledElement<E> {
    /// Creates a new styled element that wraps the given element with the given style.
    pub fn new(element: E, style: impl Into<Style>) -> StyledElement<E> {
        StyledElement {
            element,
            style: style.into(),
        }
    }
}

impl<E: Element> Element for StyledElement<E> {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        mut style: Style,
    ) -> Result<RenderResult, Error> {
        style.merge(self.style);
        self.element.render(context, area, style)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        self.element.get_probable_height(style, context, area)
    }
}

/// Adds a frame around the wrapped element.
///
/// # Examples
///
/// Direct usage:
/// ```
/// use genpdf::elements;
/// let p = elements::FramedElement::new(
///     elements::Paragraph::new("text"),
/// );
/// ```
///
/// Using [`Element::framed`][]:
/// ```
/// use genpdf::{elements, style, Element as _};
/// let p = elements::Paragraph::new("text").framed(style::LineStyle::new());
/// ```
///
/// [`Element::framed`]: ../trait.Element.html#method.framed
#[derive(Clone, Debug, Default)]
pub struct FramedElement<E: Element> {
    element: E,
    is_first: bool,
    line_style: LineStyle,
}

impl<E: Element> FramedElement<E> {
    /// Creates a new framed element that wraps the given element.
    pub fn new(element: E) -> FramedElement<E> {
        FramedElement::with_line_style(element, LineStyle::new())
    }

    /// Creates a new framed element that wraps the given element,
    /// and with the given line style.
    pub fn with_line_style(element: E, line_style: impl Into<LineStyle>) -> FramedElement<E> {
        Self {
            is_first: true,
            element,
            line_style: line_style.into(),
        }
    }
}

impl<E: Element> Element for FramedElement<E> {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        // if let Some(margins) = self.margins {
        // area.add_margins(20);
        // }
        // For the element area calculations, we have to take into account the full line thickness.
        // For the frame area, we only need half because we specify the center of the line.
        let line_thickness = self.line_style.thickness();
        let line_offset = line_thickness / 2.0;

        // Calculate the areas in which to draw the element and the frame.
        let mut element_area = area.clone();
        let mut frame_area = area.clone();
        element_area.add_margins(Margins::trbl(
            0,
            line_thickness,
            line_thickness,
            line_thickness,
        ));
        frame_area.add_margins(Margins::trbl(0, line_offset, 0, line_offset));
        if self.is_first {
            element_area.add_margins(Margins::trbl(line_thickness, 0, 0, 0));
            frame_area.add_margins(Margins::trbl(line_offset, 0, 0, 0));
        }

        // Draw the element.
        let mut result = self.element.render(context, element_area, style)?;
        result.size.width = area.size().width;
        if result.has_more {
            frame_area.set_height(result.size.height + line_offset);
        } else {
            frame_area.set_height(result.size.height + line_thickness);
        }

        // Draw the frame.

        let top_left = Position::default();
        let top_right = Position::new(frame_area.size().width, 0);
        let bottom_left = Position::new(0, frame_area.size().height);
        let bottom_right = Position::new(frame_area.size().width, frame_area.size().height);

        if self.is_first {
            result.size.height += line_thickness;
            frame_area.draw_line(
                vec![bottom_right, top_right, top_left, bottom_left],
                self.line_style,
            );
        }
        if !result.has_more {
            result.size.height += line_thickness;
            frame_area.draw_line(
                vec![top_left, bottom_left, bottom_right, top_right],
                self.line_style,
            );
        } else {
            frame_area.draw_line(vec![top_left, bottom_left], self.line_style);
            frame_area.draw_line(vec![top_right, bottom_right], self.line_style);
        }

        self.is_first = false;

        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        self.element.get_probable_height(style, context, area)
    }
}

/// An unordered list of elements with bullet points.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut list = elements::UnorderedList::new();
/// list.push(elements::Paragraph::new("first"));
/// list.push(elements::Paragraph::new("second"));
/// list.push(elements::Paragraph::new("third"));
/// ```
///
/// With setters and a custom bullet symbol:
/// ```
/// use genpdf::elements;
/// let mut list = elements::UnorderedList::with_bullet("*");
/// list.push(elements::Paragraph::new("first"));
/// list.push(elements::Paragraph::new("second"));
/// list.push(elements::Paragraph::new("third"));
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let list = elements::UnorderedList::new()
///     .element(elements::Paragraph::new("first"))
///     .element(elements::Paragraph::new("second"))
///     .element(elements::Paragraph::new("third"));
/// ```
///
/// Nested list using a [`LinearLayout`][]:
/// ```
/// use genpdf::elements;
/// let list = elements::UnorderedList::new()
///     .element(
///         elements::OrderedList::new()
///             .element(elements::Paragraph::new("Sublist with bullet point"))
///     )
///     .element(
///         elements::LinearLayout::vertical()
///             .element(elements::Paragraph::new("Sublist without bullet point:"))
///             .element(
///                 elements::OrderedList::new()
///                     .element(elements::Paragraph::new("first"))
///                     .element(elements::Paragraph::new("second"))
///             )
///     );
/// ```
///
/// [`LinearLayout`]: struct.LinearLayout.html

/// An ordered/unordered list of elements with bullet points.
pub enum UOList {
    /// unordered list
    UnorderedList(UnorderedList),
    /// order list
    OrderedList(OrderedList),
}

impl UOList {
    /// push element to list
    pub fn push<E: Element + 'static>(&mut self, element: E) {
        match self {
            UOList::OrderedList(ol) => ol.push(element),
            UOList::UnorderedList(ul) => ul.push(element),
        }
    }
    /// push list
    pub fn push_list(&mut self, target_list: UOList) {
        match target_list {
            UOList::UnorderedList(ul) => match self {
                UOList::OrderedList(ol2) => ol2.push_list(ul),
                UOList::UnorderedList(ul2) => ul2.push_list(ul),
            },
            UOList::OrderedList(mut ol) => match self {
                UOList::OrderedList(ol2) => {
                    // print bullet display
                    // println!("bullet display: {:?}", ol2.get_bullet_display());
                    match ol2.get_bullet_display() {
                        Some(display) => ol.set_prefix(Some(display)),
                        None => {}
                    }
                    // let display = &ol2.get_bullet_display();
                    // ol.set_prefix(display);
                    ol2.push_list(ol)
                }
                UOList::UnorderedList(ul2) => ul2.push_list(ol),
            },
        }
    }
}

///
pub struct UnorderedList {
    layout: LinearLayout,
    bullet: Option<String>,
    margins: Option<Margins>,
}

impl UnorderedList {
    /// Creates a new unordered list with the default bullet point symbol.
    pub fn new() -> UnorderedList {
        UnorderedList {
            layout: LinearLayout::vertical(),
            bullet: None,
            margins: None,
        }
    }

    /// Creates a new unordered list with the given bullet point symbol.
    pub fn with_bullet(bullet: impl Into<String>) -> UnorderedList {
        UnorderedList {
            layout: LinearLayout::vertical(),
            bullet: Some(bullet.into()),
            margins: None,
        }
    }

    /// Push UnorderedList/OrderedList to the list.
    pub fn push_list<E: Element + 'static>(&mut self, list: E) {
        let mut point = BulletPoint::new(list);
        point.indent = point.indent / 2.0;
        point.set_bullet("".to_string());
        self.layout.push(point);
    }

    /// Adds an element to this list.
    pub fn push<E: Element + 'static>(&mut self, element: E) {
        let mut point = BulletPoint::new(element);
        if let Some(bullet) = &self.bullet {
            point.set_bullet(bullet.clone());
        }
        self.layout.push(point);
    }

    /// Adds an element to this list and returns the list.
    pub fn element<E: Element + 'static>(mut self, element: E) -> Self {
        self.push(element);
        self
    }

    /// get margins
    pub fn get_margins(&self) -> Option<Margins> {
        self.margins
    }

    /// set margins
    pub fn set_margins(&mut self, margins: Margins) {
        self.margins = Some(margins);
    }
}

impl Element for UnorderedList {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        if let Some(margins) = self.get_margins() {
            area.add_margins(margins);
        }
        let mut result = self.layout.render(context, area, style)?;
        if let Some(margins) = self.margins {
            result.size.width += margins.left + margins.right;
            result.size.height += margins.top + margins.bottom;
        }
        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        let mut height = self.layout.get_probable_height(style, context, area);
        if let Some(margins) = self.get_margins() {
            height += margins.top + margins.bottom;
        }
        height
    }
}

impl Default for UnorderedList {
    fn default() -> UnorderedList {
        UnorderedList::new()
    }
}

impl<E: Element + 'static> iter::Extend<E> for UnorderedList {
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        for element in iter {
            self.push(element);
        }
    }
}

impl<E: Element + 'static> iter::FromIterator<E> for UnorderedList {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let mut list = Self::default();
        list.extend(iter);
        list
    }
}

/// An ordered list of elements with arabic numbers.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut list = elements::OrderedList::new();
/// list.push(elements::Paragraph::new("first"));
/// list.push(elements::Paragraph::new("second"));
/// list.push(elements::Paragraph::new("third"));
/// ```
///
/// With setters and a custom start number:
/// ```
/// use genpdf::elements;
/// let mut list = elements::OrderedList::with_start(5);
/// list.push(elements::Paragraph::new("first"));
/// list.push(elements::Paragraph::new("second"));
/// list.push(elements::Paragraph::new("third"));
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let list = elements::OrderedList::new()
///     .element(elements::Paragraph::new("first"))
///     .element(elements::Paragraph::new("second"))
///     .element(elements::Paragraph::new("third"));
/// ```
///
/// Nested list using a [`LinearLayout`][]:
/// ```
/// use genpdf::elements;
/// let list = elements::OrderedList::new()
///     .element(
///         elements::UnorderedList::new()
///             .element(elements::Paragraph::new("Sublist with number"))
///     )
///     .element(
///         elements::LinearLayout::vertical()
///             .element(elements::Paragraph::new("Sublist without number:"))
///             .element(
///                 elements::UnorderedList::new()
///                     .element(elements::Paragraph::new("first"))
///                     .element(elements::Paragraph::new("second"))
///             )
///     );
/// ```

/// [`LinearLayout`]: struct.LinearLayout.html
pub struct OrderedList {
    layout: LinearLayout,
    number: usize,
    margins: Option<Margins>,
    bullet_style: Option<Style>,
    element_spacing: Mm,
    bullet_display: Option<String>,
    prefix: Option<String>,
    // parent_bullet_display: Option<String>,
}

impl OrderedList {
    /// Creates a new ordered list starting at 1.
    pub fn new() -> OrderedList {
        OrderedList::with_start(1)
    }

    /// Creates a new ordered list with the given start number.
    pub fn with_start(start: usize) -> OrderedList {
        OrderedList {
            layout: LinearLayout::vertical(),
            number: start,
            margins: None,
            bullet_style: None,
            element_spacing: Mm(0.0),
            bullet_display: None,
            prefix: None,
            // parent_bullet_display: None,
        }
    }

    /// bullet_margins
    pub fn set_element_spacing(&mut self, element_spacing: Mm) {
        self.element_spacing = element_spacing;
    }

    /// set list_item_margin
    pub fn set_list_item_spacing(&mut self, spacing: f64) {
        self.layout.set_list_item_spacing(spacing)
    }

    /// get list_item_margin
    // pub fn get_list_item_margin(&self) -> Option<Margins> {
    //     // self.list_item_margin.clone()
    //     self.layout.get_list_item_margins()
    // }

    /// set prefix
    pub fn set_prefix(&mut self, prefix: Option<String>) {
        self.prefix = prefix;
    }

    /// get prefix
    pub fn get_prefix(&self) -> Option<String> {
        self.prefix.clone()
    }

    /// get bullet display
    pub fn get_bullet_display(&self) -> Option<String> {
        self.bullet_display.clone()
    }

    /// Push OrderedList/UnordredList to the list.
    pub fn push_list<E: Element + 'static>(&mut self, list: E) {
        let mut point = BulletPoint::new(list);
        // point.indent = Mm(0.0); //point.indent / 2.0;
        // point.bullet_space = Mm(0.0);
        point.set_bullet("".to_string());
        // point.set_bullet_prefix(parent_bullet_display);
        self.layout.push(point);
    }

    /// Adds an element to this list.
    pub fn push<E: Element + 'static>(&mut self, element: E) {
        let mut point = BulletPoint::new(element);
        let bullet = match self.get_prefix() {
            Some(mut prefix) => {
                if !prefix.ends_with(".") {
                    prefix = format!("{}.", prefix);
                }
                format!("{}{}", prefix, self.number)
            }
            None => format!("{}.", self.number),
        };

        self.bullet_display = Some(bullet.to_owned());
        point.set_bullet(bullet);
        point.set_style(self.bullet_style);
        // point.set_margins(margins);
        self.layout.push(point);
        self.number += 1;
    }

    /// Adds an element to this list and returns the list.
    pub fn element<E: Element + 'static>(mut self, element: E) -> Self {
        self.push(element);
        self
    }

    /// get margins
    pub fn get_margins(&self) -> Option<Margins> {
        self.margins
    }

    /// set margins
    pub fn set_margins(&mut self, margins: Margins) {
        self.margins = Some(margins);
    }

    /// set bullet style
    pub fn set_bullet_style(&mut self, style: Style) {
        self.bullet_style = Some(style);
    }

    /// get bullet style
    pub fn get_bullet_style(&self) -> Option<Style> {
        self.bullet_style
    }
}

impl Element for OrderedList {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        if let Some(margins) = self.get_margins() {
            area.add_margins(margins);
        }
        let mut result = self.layout.render(context, area, style)?;
        if let Some(margins) = self.margins {
            result.size.width += margins.left + margins.right;
            result.size.height += margins.top + margins.bottom;
        }
        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        let mut height = self.layout.get_probable_height(style, context, area);
        if let Some(margins) = self.get_margins() {
            height += margins.top + margins.bottom;
        }
        height
    }
}

impl Default for OrderedList {
    fn default() -> OrderedList {
        OrderedList::new()
    }
}

impl<E: Element + 'static> iter::Extend<E> for OrderedList {
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        for element in iter {
            self.push(element);
        }
    }
}

impl<E: Element + 'static> iter::FromIterator<E> for OrderedList {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let mut list = Self::default();
        list.extend(iter);
        list
    }
}

/// A bullet point in a list.
///
/// This is a helper element for the [`OrderedList`][] and [`UnorderedList`][] types, but you can
/// also use it directly if you have special requirements.
///
/// # Example
///
/// ```
/// use genpdf::elements;
/// let layout = elements::LinearLayout::vertical()
///     .element(elements::BulletPoint::new(elements::Paragraph::new("first"))
///         .with_bullet("a)"))
///     .element(elements::BulletPoint::new(elements::Paragraph::new("second"))
///         .with_bullet("b)"));
/// ```
///
/// [`OrderedList`]: struct.OrderedList.html
/// [`UnorderedList`]: struct.UnorderedList.html
pub struct BulletPoint<E: Element> {
    element: E,
    indent: Mm,
    bullet_space: Mm,
    bullet: String,
    bullet_rendered: bool,
    style: Option<Style>,
    margins: Option<Margins>,
    bullet_prefix: Option<String>,
}

impl<E: Element> BulletPoint<E> {
    /// Creates a new bullet point with the given element.
    pub fn new(element: E) -> BulletPoint<E> {
        BulletPoint {
            element,
            indent: Mm::from(10),
            bullet_space: Mm::from(2),
            bullet: String::from("–"),
            bullet_rendered: false,
            style: None,
            margins: None,
            bullet_prefix: None,
        }
    }

    /// set bullet style
    pub fn set_style(&mut self, style: Option<Style>) {
        self.style = style;
    }

    /// Sets the bullet point symbol for this bullet point.
    pub fn set_bullet(&mut self, bullet: impl Into<String>) {
        self.bullet = bullet.into();
    }

    /// Sets the bullet point prefix
    pub fn set_bullet_prefix(&mut self, prefix: Option<String>) {
        self.bullet_prefix = prefix;
    }

    /// Sets the bullet point symbol for this bullet point and returns the bullet point.
    pub fn with_bullet(mut self, bullet: impl Into<String>) -> Self {
        self.set_bullet(bullet);
        self
    }

    /// set margins
    pub fn set_margins(&mut self, margins: Option<Margins>) {
        self.margins = margins;
    }
}

impl<E: Element> Element for BulletPoint<E> {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        // if let Some(element_spacing) = self.element
        // area.add_margins(Margins::trbl(10, 0, 0, 0));
        if let Some(mr) = self.margins {
            area.add_margins(mr);
        }
        let mut element_area = area.clone();
        element_area.add_offset(Position::new(self.indent, 0));

        let mut result = self.element.render(context, element_area, style)?;
        result.size.width += self.indent;
        if !self.bullet_rendered {
            // println!("Bullet self.style: {:?}", self.style);
            // println!("Bullet style: {:?}", style);
            let style = match self.style {
                Some(s) => style.and(s),
                None => style,
            };
            // println!("Bullet final style: {:?}", style);

            let bullet_width = style.str_width(&context.font_cache, &self.bullet);
            let x = self.indent - bullet_width - self.bullet_space;
            area.print_str(
                &context.font_cache,
                Position::new(x, 0),
                style,
                &self.bullet,
            )?;

            if style.is_underline() {
                let ls = LineStyle::new().with_thickness(0.2);
                let left = x;
                let right = left + bullet_width;
                let line_offset = ls.thickness() / 2.0;
                let bottom = style.metrics(&context.font_cache).line_height;
                let bottom_points = vec![
                    Position::new(left, bottom - line_offset),
                    Position::new(right, bottom - line_offset),
                ];
                area.draw_line(bottom_points, ls);
                result.size.height += ls.thickness();
            }
            self.bullet_rendered = true;
        }
        if let Some(mr) = self.margins {
            result.size.height += mr.top + mr.bottom;
        }
        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        self.element.get_probable_height(style, context, area)
    }
}

/// A decorator for table cells.
///
/// Implementations of this trait can be used to style cells of a [`TableLayout`][].
///
/// [`TableLayout`]: struct.TableLayout.html
pub trait CellDecorator {
    /// Sets the size of the table.
    ///
    /// This function is called once before the first call to [`prepare_cell`][] or
    /// [`decorate_cell`][].
    ///
    /// [`prepare_cell`]: #tymethod.prepare_cell
    /// [`decorate_cell`]: #tymethod.decorate_cell
    fn set_table_size(&mut self, num_columns: usize, num_rows: usize) {
        let _ = (num_columns, num_rows);
    }

    /// Prepares the cell with the given indizes and returns the area for rendering the cell.
    fn prepare_cell<'p>(
        &self,
        column: usize,
        row: usize,
        area: render::Area<'p>,
    ) -> render::Area<'p> {
        let _ = (column, row);
        area
    }

    /// Styles the cell with the given indizes thas has been rendered within the given area and the
    /// given row height and return the total row height.
    fn decorate_cell(
        &mut self,
        column: usize,
        row: usize,
        has_more: bool,
        area: render::Area<'_>,
        row_height: Mm,
        bg_color: Option<style::Color>,
    ) -> Mm;
}

/// A cell decorator that draws frames around table cells.
///
/// This decorator draws frames around the cells of a [`TableLayout`][].  You can configure whether
/// inner, outer and continuation borders are drawn.  A continuation border is a border between a
/// cell and the page margin that occurs if a cell has to be wrapped to a new page.
///
/// [`TableLayout`]: struct.TableLayout.html
#[derive(Clone, Debug, Default)]
pub struct FrameCellDecorator {
    inner: bool,
    outer: bool,
    // cont: bool,
    line_style: LineStyle,
    num_columns: usize,
    num_rows: usize,
    last_row: Option<usize>,
}

impl FrameCellDecorator {
    /// Creates a new frame cell decorator with the given settings for inner, outer and
    /// continuation borders.
    pub fn new(inner: bool, outer: bool) -> FrameCellDecorator {
        FrameCellDecorator {
            inner,
            outer,
            // cont,
            ..Default::default()
        }
    }

    /// Creates a new frame cell decorator with the given border settings, as well as a line style.
    pub fn with_line_style(
        inner: bool,
        outer: bool,
        // cont: bool,
        line_style: impl Into<LineStyle>,
    ) -> FrameCellDecorator {
        Self {
            inner,
            outer,
            // cont,
            line_style: line_style.into(),
            ..Default::default()
        }
    }

    fn print_left(&self, column: usize) -> bool {
        if column == 0 {
            self.outer
        } else {
            self.inner
        }
    }

    fn print_right(&self, column: usize) -> bool {
        if column + 1 == self.num_columns {
            self.outer
        } else {
            false
        }
    }

    fn print_top(&self, row: usize, has_more: bool) -> bool {
        if has_more {
            self.outer
        } else if self.last_row.map(|last_row| row > last_row).unwrap_or(true) {
            if row == 0 {
                self.outer
            } else {
                self.inner
            }
        } else {
            // self.cont
            true
        }
    }

    fn print_bottom(&self, row: usize, has_more: bool) -> bool {
        if has_more {
            // self.cont
            true
        } else if row + 1 == self.num_rows {
            self.outer
        } else {
            false
        }
    }
}

impl CellDecorator for FrameCellDecorator {
    fn set_table_size(&mut self, num_columns: usize, num_rows: usize) {
        self.num_columns = num_columns;
        self.num_rows = num_rows;
    }

    fn prepare_cell<'p>(
        &self,
        column: usize,
        row: usize,
        mut area: render::Area<'p>,
    ) -> render::Area<'p> {
        let margin = self.line_style.thickness();
        let margins = Margins::trbl(
            if self.print_top(row, false) {
                margin
            } else {
                0.into()
            },
            if self.print_right(column) {
                margin
            } else {
                // Fix to avoid a gap betwen the right border and the next cell
                area.set_width(area.size().width + margin);
                0.into()
            },
            if self.print_bottom(row, false) {
                margin
            } else {
                0.into()
            },
            if self.print_left(column) {
                margin
            } else {
                0.into()
            },
        );
        area.add_margins(margins);
        area
    }

    fn decorate_cell(
        &mut self,
        column: usize,
        row: usize,
        has_more: bool,
        area: render::Area<'_>,
        row_height: Mm,
        bg_color: Option<style::Color>,
    ) -> Mm {
        let print_top = self.print_top(row, has_more);
        let print_bottom = self.print_bottom(row, has_more);
        let print_left = self.print_left(column);
        let print_right = self.print_right(column);

        // println!("----------------------------------------------------------------------------------------------------------------------------------------");
        // println!(
        //     "Cell: {},{}: top={}, bottom={}, left={}, right={}",
        //     column, row, print_top, print_bottom, print_left, print_right
        // );
        // println!("----------------------------------------------------------------------------------------------------------------------------------------");

        let size = area.size();
        let line_offset = self.line_style.thickness() / 2.0;

        let left = Mm::from(0);
        let right = size.width;
        let top = Mm::from(0);
        let bottom = row_height
            + if print_bottom {
                self.line_style.thickness()
            } else {
                0.into()
            }
            + if print_top {
                self.line_style.thickness()
            } else {
                0.into()
            };

        if let Some(color) = bg_color {
            let bottom_left = Position::new(left + line_offset, bottom - line_offset);
            let top_left = Position::new(left + line_offset, top + line_offset);
            let top_right = Position::new(right - line_offset, top + line_offset);
            let bottom_right = Position::new(right - line_offset, bottom - line_offset);

            // println!("decorateCell bottom_left: {:?}", bottom_left);
            // println!("decorateCell top_left: {:?}", top_left);
            // println!("decorateCell top_right: {:?}", top_right);
            // println!("decorateCell bottom_right: {:?}", bottom_right);
            let filled_shape_points = vec![bottom_left, top_left, top_right, bottom_right];
            // println!("----------------------------------------------------------------------------------------------------------------------------------------");
            // println!(
            //     "decorateCell, filled_shape_points: {:?}",
            //     filled_shape_points
            // );
            // println!("----------------------------------------------------------------------------------------------------------------------------------------");
            area.draw_filled_shape(filled_shape_points, Some(color), self.line_style);
        }

        let mut total_height = row_height;

        let top_points = vec![
            Position::new(left, top + line_offset),
            Position::new(right, top + line_offset),
        ];
        if print_top {
            // println!("decorateCell, top_points: {:?}", top_points);
            area.draw_line(top_points, self.line_style);
            total_height += self.line_style.thickness();
        }
        let right_points = vec![
            Position::new(right - line_offset, top),
            Position::new(right - line_offset, bottom),
        ];

        if print_right {
            // println!("----------------------------------------------------------------------------------------------------------------------------------------");
            // println!("decorateCell, right_points: {:?}", right_points);
            // println!("----------------------------------------------------------------------------------------------------------------------------------------");
            area.draw_line(right_points, self.line_style);
        }

        let bottom_points = vec![
            Position::new(left, bottom - line_offset),
            Position::new(right, bottom - line_offset),
        ];
        if print_bottom {
            // println!("----------------------------------------------------------------------------------------------------------------------------------------");
            // println!("decorateCell, bottom_points: {:?}", bottom_points);
            // println!("----------------------------------------------------------------------------------------------------------------------------------------");
            area.draw_line(bottom_points, self.line_style);
            total_height += self.line_style.thickness();
        }

        let left_points = vec![
            Position::new(left + line_offset, top),
            Position::new(left + line_offset, bottom),
        ];
        // println!("decorateCell, left_points: {:?}", left_points);
        if print_left {
            area.draw_line(left_points, self.line_style);
        }

        if column + 1 == self.num_columns {
            self.last_row = Some(row);
        }

        total_height
    }
}

/// A row of a table layout.
///
/// This is a helper struct for populating a [`TableLayout`][].  After you have added all elements
/// to the row using [`push_element`][] or [`element`][], you can append the row to the table
/// layout by calling [`push`][].
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut table = elements::TableLayout::new(vec![1, 1]);
/// let mut row = table.row();
/// row.push_element(elements::Paragraph::new("Cell 1"));
/// row.push_element(elements::Paragraph::new("Cell 2"));
/// row.push().expect("Invalid table row");
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let table = elements::TableLayout::new(vec![1, 1])
///     .row()
///     .element(elements::Paragraph::new("Cell 1"))
///     .element(elements::Paragraph::new("Cell 2"))
///     .push()
///     .expect("Invalid table row");
/// ```
///
/// [`TableLayout`]: struct.TableLayout.html
/// [`push`]: #method.push
/// [`push_element`]: #method.push_element
/// [`element`]: #method.element
pub struct TableLayoutRow<'a> {
    table_layout: &'a mut TableLayout,
    cells: Vec<TableCell>,
}

/// A cell of a table layout.
pub struct TableCell {
    element: Box<dyn Element>,
    background_color: Option<style::Color>,
    draw_left_border: bool,
    draw_right_border: bool,
    draw_top_border: bool,
    draw_bottom_border: bool,
}

impl TableCell {
    /// new
    pub fn new(element: Box<dyn Element>, background_color: Option<style::Color>) -> TableCell {
        TableCell {
            element,
            background_color,
            draw_left_border: true,
            draw_right_border: true,
            draw_top_border: true,
            draw_bottom_border: true,
        }
    }

    /// set draw_left_border
    pub fn draw_left_border(mut self, draw_left_border: bool) -> Self {
        self.draw_left_border = draw_left_border;
        self
    }

    /// set draw_right_border
    pub fn draw_right_border(mut self, draw_right_border: bool) -> Self {
        self.draw_right_border = draw_right_border;
        self
    }

    /// set draw_top_border
    pub fn draw_top_border(mut self, draw_top_border: bool) -> Self {
        self.draw_top_border = draw_top_border;
        self
    }

    /// set draw_bottom_border
    pub fn draw_bottom_border(mut self, draw_bottom_border: bool) -> Self {
        self.draw_bottom_border = draw_bottom_border;
        self
    }
}

impl<'a> TableLayoutRow<'a> {
    fn new(table_layout: &'a mut TableLayout) -> TableLayoutRow<'a> {
        TableLayoutRow {
            table_layout,
            cells: Vec::new(),
        }
    }

    /// Create a cell with  given element and color and add to cells
    pub fn cell<E: IntoBoxedElement>(mut self, element: E, color: Option<style::Color>) -> Self {
        self.cells.push(TableCell {
            element: element.into_boxed_element(),
            background_color: color,
            draw_left_border: true,
            draw_right_border: true,
            draw_top_border: true,
            draw_bottom_border: true,
        });
        self
    }

    /// Tries to append this row to the table.
    ///
    /// This method fails if the number of elements in this row does not match the number of
    /// columns in the table.
    pub fn push(self) -> Result<(), Error> {
        self.table_layout.push_row(self.cells, None)
    }
}

/// Arranges elements in columns and rows.
///
/// This struct can be used to layout arbitrary elements in columns in rows, or to draw typical
/// tables.  You can customize the cell style by providing a [`CellDecorator`][] implementation.
/// If you want to print a typical table with borders around the cells, use the
/// [`FrameCellDecorator`][].
///
/// The column widths are determined by the weights that have been set in the constructor.  The
/// table always uses the full width of the provided area.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut table = elements::TableLayout::new(vec![1, 1]);
/// table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));
/// let mut row = table.row();
/// row.push_element(elements::Paragraph::new("Cell 1"));
/// row.push_element(elements::Paragraph::new("Cell 2"));
/// row.push().expect("Invalid table row");
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let table = elements::TableLayout::new(vec![1, 1])
///     .row()
///     .element(elements::Paragraph::new("Cell 1"))
///     .element(elements::Paragraph::new("Cell 2"))
///     .push()
///     .expect("Invalid table row");
/// ```
///
/// [`CellDecorator`]: trait.CellDecorator.html
/// [`FrameCellDecorator`]: struct.FrameCellDecorator.html
///
#[derive(Clone)]
pub enum ColumnWidths {
    /// The columns have the given weights.
    Weights(Vec<usize>),
    /// The columns have the given pixel widths.
    PixelWidths(Vec<f64>),
}

impl ColumnWidths {
    /// Returns the number of columns.
    pub fn len(&self) -> usize {
        match self {
            ColumnWidths::Weights(weights) => weights.len(),
            ColumnWidths::PixelWidths(widths) => widths.len(),
        }
    }

    /// Returns size of the total columns.
    pub fn is_empty(&self) -> bool {
        match self {
            ColumnWidths::Weights(weights) => weights.is_empty(),
            ColumnWidths::PixelWidths(widths) => widths.is_empty(),
        }
    }

    /// to_vec
    pub fn to_vec(&self) -> Vec<f64> {
        match self {
            ColumnWidths::Weights(weights) => {
                let mut widths = Vec::new();
                for i in 0..weights.len() {
                    widths.push(weights[i] as f64);
                }
                widths
            }
            ColumnWidths::PixelWidths(widths) => widths.clone(),
        }
    }
}

/// Table Row
pub struct TableRow {
    cells: Vec<TableCell>,
    row_height: Option<i32>,
}

/// Table Layout
pub struct TableLayout {
    column_weights: ColumnWidths,
    rows: Vec<TableRow>,
    render_idx: usize,
    cell_decorator: Option<Box<dyn CellDecorator>>,
    header_row_callback_fn: Option<TableHeaderRowCallback>,
    draw_inner_borders: bool,
    draw_outer_borders: bool,
    has_header_row_callback: bool,
    margins: Option<Margins>,
}

type TableHeaderRowCallback = Box<dyn Fn(usize) -> Result<Box<dyn Element>, Error>>;

impl TableLayout {
    // /// Return column weights
    ///
    pub fn column_weights(&self) -> ColumnWidths {
        self.column_weights.clone()
    }

    // /// Return draw_inner_borders, draw_outer_borders
    ///
    pub fn borders(&self) -> (bool, bool) {
        (self.draw_inner_borders, self.draw_outer_borders)
    }

    /// Creates a new table layout with the given column weights.
    ///
    pub fn new(column_weights: ColumnWidths) -> Self {
        TableLayout::new_with_borders(column_weights, false, false)
    }

    /// Creates a new table layout with the given column weights.
    ///
    /// The column weights are used to determine the relative width of the columns.  The number of
    /// column weights determines the number of columns in the table.
    pub fn new_with_borders(
        column_weights: ColumnWidths,
        draw_inner_borders: bool,
        draw_outer_borders: bool,
    ) -> TableLayout {
        let mut tl = TableLayout {
            column_weights,
            rows: Vec::new(),
            render_idx: 0,
            cell_decorator: None,
            header_row_callback_fn: None,
            draw_inner_borders,
            draw_outer_borders,
            has_header_row_callback: false,
            margins: None,
        };
        set_cell_decorator(&mut tl, draw_inner_borders, draw_outer_borders);
        tl
    }

    /// set margins
    /// margins is the distance between the text and the border
    pub fn set_margins(&mut self, margins: Margins) {
        self.margins = Some(margins);
    }

    /// returns the current padding
    pub fn get_margins(&self) -> Option<Margins> {
        self.margins
    }

    /// get has header row callback
    ///
    pub fn has_header_row_callback(&self) -> bool {
        self.has_header_row_callback
    }
    /// set has header row callback
    ///
    pub fn set_has_header_row_callback(&mut self, has_header_row_callback: bool) {
        self.has_header_row_callback = has_header_row_callback;
    }

    /// register header row callback
    pub fn register_header_row_callback_fn<F, E>(&mut self, cb: F)
    where
        F: Fn(usize) -> Result<E, Error> + 'static,
        E: Element + 'static,
    {
        self.header_row_callback_fn =
            Some(Box::new(move |page| cb(page).map(|e| Box::new(e) as _)));
    }

    /// Sets the cell decorator for this table.
    pub fn set_cell_decorator(&mut self, decorator: impl CellDecorator + 'static) {
        self.cell_decorator = Some(Box::from(decorator));
    }

    /// Adds a row to this table using the [`TableLayoutRow`][] helper struct.
    ///
    /// [`TableLayoutRow`]: struct.TableLayoutRow.html
    pub fn row(&mut self) -> TableLayoutRow<'_> {
        TableLayoutRow::new(self)
    }

    /// Adds a row to this table.
    ///
    /// The number of elements in the given vector must match the number of columns.  Otherwise, an
    /// error is returned.
    pub fn push_row(
        &mut self,
        cells: Vec<TableCell>,
        row_height: Option<i32>,
    ) -> Result<(), Error> {
        if cells.len() == self.column_weights.len() {
            let r = TableRow { cells, row_height };
            self.rows.push(r);
            Ok(())
        } else {
            Err(Error::new(
                format!(
                    "Expected {} elements in table row, received {}",
                    self.column_weights.len(),
                    cells.len()
                ),
                ErrorKind::InvalidData,
            ))
        }
    }

    fn render_row(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        let areas = area.split_horizontally(&self.column_weights);
        let cell_areas = if let Some(decorator) = &self.cell_decorator {
            areas
                .iter()
                .enumerate()
                .map(|(i, area)| decorator.prepare_cell(i, self.render_idx, area.clone()))
                .collect()
        } else {
            areas.clone()
        };

        // get row probable height
        let mut row_probable_height = Mm::from(0);
        for (area, cell) in cell_areas
            .clone()
            .iter()
            .zip(self.rows[self.render_idx].cells.iter_mut())
        {
            let el_probable_height = cell
                .element
                .get_probable_height(style, context, area.clone());
            row_probable_height = row_probable_height.max(el_probable_height);
        }
        if let Some(rh) = self.rows[self.render_idx].row_height {
            if rh > row_probable_height.0 as i32 {
                row_probable_height = rh.into();
            }
        }
        if row_probable_height > area.size().height {
            result.has_more = true;
            return Ok(result);
        }

        if let Some(decorator) = &mut self.cell_decorator {
            for (i, area) in cell_areas.clone().into_iter().enumerate() {
                let cell_bg_color = self.rows[self.render_idx].cells[i].background_color;
                let height = decorator.decorate_cell(
                    i,
                    self.render_idx,
                    true,
                    area,
                    row_probable_height,
                    cell_bg_color,
                );
                result.size.height = result.size.height.max(height);
            }
        }

        let mut row_height = Mm::from(0);
        for (area, cell) in cell_areas
            .iter()
            .zip(self.rows[self.render_idx].cells.iter_mut())
        {
            let element_result = cell.element.render(context, area.clone(), style)?;
            result.has_more |= element_result.has_more;
            row_height = row_height.max(element_result.size.height);
        }
        result.size.height = row_height;
        if let Some(rh) = self.rows[self.render_idx].row_height {
            if rh > row_height.0 as i32 {
                result.size.height = rh.into();
            }
        }
        Ok(result)
    }
}

fn set_cell_decorator(tl: &mut TableLayout, draw_inner_borders: bool, draw_outer_borders: bool) {
    tl.set_cell_decorator(FrameCellDecorator::new(
        draw_inner_borders,
        draw_outer_borders,
        // false,
    ));
}

impl Element for TableLayout {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        if self.column_weights.is_empty() {
            return Ok(result);
        }
        if let Some(margins) = self.margins {
            result.size.height += margins.top + margins.bottom;
            area.add_margins(margins);
        }
        if let Some(decorator) = &mut self.cell_decorator {
            decorator.set_table_size(self.column_weights.len(), self.rows.len());
        }
        result.size.width = area.size().width;

        // render table header row using callback function
        if let Some(cb) = &self.header_row_callback_fn {
            let rr = match cb(context.page_number) {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            };
            match rr {
                Ok(mut element) => {
                    let prob_height = element.get_probable_height(style, context, area.clone());
                    if prob_height > area.size().height {
                        log(
                            "TableHeaderRowSpace",
                            "Cannot render header row, not enough space",
                        );
                        result.has_more = true;
                        return Ok(result);
                    }
                    let header_result = element.render(context, area.clone(), style)?;
                    result.size.height += header_result.size.height;
                    area.add_offset(Position::new(0, header_result.size.height));
                }
                Err(e) => {
                    return Err(e);
                }
            };
        };

        while self.render_idx < self.rows.len() {
            let row_result = self.render_row(context, area.clone(), style)?;
            result.size.height += row_result.size.height;
            area.add_offset(Position::new(0, row_result.size.height));
            if row_result.has_more {
                break;
            }
            self.render_idx += 1;
        }
        result.has_more = self.render_idx < self.rows.len();
        Ok(result)
    }

    fn get_probable_height(
        &mut self,
        style: style::Style,
        context: &Context,
        area: render::Area<'_>,
    ) -> Mm {
        let mut height = Mm::from(0);
        // calculate table height using rows
        for row in self.rows.iter_mut() {
            let mut row_height = Mm::from(0);
            for cell in row.cells.iter_mut() {
                let cell_height = cell
                    .element
                    .get_probable_height(style, context, area.clone());
                row_height = row_height.max(cell_height);
            }
            height += row_height;
        }

        // TODO: calculate table height row height
        if let Some(cb) = &self.header_row_callback_fn {
            let rr = match cb(context.page_number) {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            };
            match rr {
                Ok(mut element) => {
                    let header_height = element.get_probable_height(style, context, area.clone());
                    height += header_height;
                }
                Err(_) => {
                    return Mm::from(0);
                }
            };
        };
        match self.margins {
            Some(margins) => {
                height += margins.top + margins.bottom;
            }
            None => {}
        }
        height
    }
}
