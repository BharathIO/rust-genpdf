// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: CC0-1.0

//! This example generates a demo PDF document and writes it to the path that was passed as the
//! first command-line argument.  You may have to adapt the `FONT_DIRS`, `DEFAULT_FONT_NAME` and
//! `MONO_FONT_NAME` constants for your system so that these files exist:
//! - `{FONT_DIR}/{name}-Regular.ttf`
//! - `{FONT_DIR}/{name}-Bold.ttf`
//! - `{FONT_DIR}/{name}-Italic.ttf`
//! - `{FONT_DIR}/{name}-BoldItalic.ttf`
//! for `name` in {`DEFAULT_FONT_NAME`, `MONO_FONT_NAME`}.
//!
//! The generated document using the latest `genpdf-rs` release is available
//! [here](https://genpdf-rs.ireas.org/examples/demo.pdf).

use std::env;
use std::path::PathBuf;

use genpdf::error::Error;
use genpdf::fonts::from_files;
use genpdf::fonts::FontData;
use genpdf::fonts::FontFamily;
use genpdf::style::get_color;
use genpdf::Alignment;
use genpdf::Element as _;
use genpdf::{elements, style};

// const DEFAULT_FONT_NAME: &'static str = "LiberationSans";
const MONO_FONT_NAME: &'static str = "ArialUnicode";
const LOREM_IPSUM: &'static str =
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut \
    labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco \
    laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in \
    voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat \
    non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

// const LOREM_IPSUM: &'static str =
//     "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut";

fn get_default_font_dir() -> PathBuf {
    let mut font_dir = PathBuf::new();
    let dir = env::current_dir().ok();
    if let Some(dir) = dir {
        font_dir = dir.join("/Users/bharath/Work/Fonts/");
    }
    font_dir
}

const DEFAULT_FONT_NAME: &str = "OpenSans";
pub fn get_default_font() -> Result<FontFamily<FontData>, Error> {
    get_font(DEFAULT_FONT_NAME)
}

pub fn get_font(font_name: &str) -> Result<FontFamily<FontData>, Error> {
    let font_dir = get_default_font_dir();
    println!("Font dir: {}", font_dir.display());
    let font = match from_files(font_dir, font_name, None) {
        Ok(f) => f,
        Err(e) => {
            let err = format!("Failed to load font: {}", e);
            println!("{}", err);
            return Err(e);
        }
    };
    Ok(font)
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 1 {
        panic!("Missing argument: output file");
    }
    let output_file = &args[0];

    let default_font = match get_default_font() {
        Ok(f) => f,
        Err(e) => {
            let err = format!("Failed to load default font: {}", e);
            println!("{}", err);
            return;
        }
    };

    let monospace_font = match get_font(MONO_FONT_NAME) {
        Ok(f) => f,
        Err(e) => {
            let err = format!("Failed to load mono font: {}", e);
            println!("{}", err);
            return;
        }
    };

    // let monospace_font = fonts::from_files(font_dir, MONO_FONT_NAME, Some(fonts::Builtin::Courier))
    //     .expect("Failed to load the monospace font family");
    let mut doc = genpdf::Document::new(default_font);
    doc.set_title("genpdf Demo Document");
    doc.set_minimal_conformance();
    // doc.set_line_spacing(1.5);

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    decorator.set_header(|page| {
        println!("Page: {}", page);
        get_header_widget()
    });
    doc.set_page_decorator(decorator);

    #[cfg(feature = "hyphenation")]
    {
        use hyphenation::Load;

        doc.set_hyphenator(
            hyphenation::Standard::from_embedded(hyphenation::Language::EnglishUS)
                .expect("Failed to load hyphenation data"),
        );
    }

    let monospace = doc.add_font_family(monospace_font);
    let code = style::Style::from(monospace).bold();
    let red = style::Color::Rgb(255, 0, 0);
    let blue = style::Color::Rgb(0, 0, 255);

    doc.push(
        elements::Paragraph::new("genpdf Demo Document")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(20)),
    );
    doc.push(elements::Break::new(1.5));
    doc.push(elements::Paragraph::new(
        "This document demonstrates how the genpdf crate generates PDF documents. Currently, \
         genpdf supports these elements:",
    ));

    let mut list = elements::UnorderedList::new();
    list.push(
        elements::Paragraph::default()
            .styled_string("Text", code)
            .string(", a single line of formatted text without wrapping."),
    );
    list.push(
        elements::Paragraph::default()
            .styled_string("Paragraph", code)
            .string(
                ", one or more lines of formatted text with wrapping and an alignment (left, \
                 center, right).",
            ),
    );
    list.push(
        elements::Paragraph::default()
            .styled_string("FramedElement", code)
            .string(", a frame drawn around other elements."),
    );
    list.push(
        elements::Paragraph::default()
            .styled_string("PaddedElement", code)
            .string(", an element with an additional padding."),
    );
    list.push(
        elements::Paragraph::default()
            .styled_string("StyledElement", code)
            .string(", an element with new default style."),
    );

    list.push(
        elements::Paragraph::default()
            .styled_string("UnorderedList", code)
            .string(", an unordered list of bullet points."),
    );

    list.push(
        elements::LinearLayout::vertical()
            .element(
                elements::Paragraph::default()
                    .styled_string("OrderedList", code)
                    .string(", an ordered list of bullet points."),
            )
            .element(
                elements::OrderedList::new()
                    .element(elements::Paragraph::new("Just like this."))
                    .element(elements::Paragraph::new("And this.")),
            ),
    );

    list.push(
        elements::LinearLayout::vertical()
            .element(
                elements::Paragraph::default()
                    .styled_string("BulletPoint", code)
                    .string(", an element with a bullet point, just like in this list."),
            )
            .element(elements::BulletPoint::new(elements::Paragraph::new(
                "Of course, lists can also be nested.",
            )))
            .element(
                elements::BulletPoint::new(elements::Paragraph::new(
                    "And you can change the bullet symbol.",
                ))
                .with_bullet("•"),
            ),
    );

    list.push(
        elements::Paragraph::default()
            .styled_string("LinearLayout", code)
            .string(
                ", a container that vertically stacks its elements. The root element of a \
                 document is always a LinearLayout.",
            ),
    );
    list.push(
        elements::Paragraph::default()
            .styled_string("TableLayout", code)
            .string(", a container that arranges its elements in rows and columns."),
    );
    list.push(elements::Paragraph::new("And some more utility elements …"));
    doc.push(list);
    doc.push(elements::Break::new(1.5));

    doc.push(elements::Paragraph::new(
        "You already saw lists and formatted centered text. Here are some other examples:",
    ));
    doc.push(elements::Paragraph::new("This is right-aligned text.").aligned(Alignment::Right));
    doc.push(
        elements::Paragraph::new("And this paragraph has a frame drawn around it and is colored.")
            .padded(genpdf::Margins::vh(0, 1))
            .framed(style::LineStyle::from(style::Color::Rgb(0, 0, 255)).with_thickness(0.3))
            .styled(red),
    );
    doc.push(
        elements::Paragraph::new("You can also use other fonts if you want to.").styled(monospace),
    );
    doc.push(
        elements::Paragraph::default()
            .string("You can also ")
            .styled_string("combine ", red)
            .styled_string("multiple ", style::Style::from(blue).italic())
            .styled_string("formats", code)
            .string(" in one paragraph.")
            .styled(style::Style::new().with_font_size(16)),
    );
    doc.push(elements::Break::new(1.5));

    doc.push(elements::Paragraph::new(
        "Embedding images also works using the 'images' feature.",
    ));
    #[cfg(feature = "images")]
    images::do_image_test(&mut doc);

    doc.push(elements::Paragraph::new("Here is an example table:"));

    let mut table =
        elements::TableLayout::new(elements::ColumnWidths::PixelWidths(vec![30.0, 100.0]));
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true));
    table
        .row()
        .cell(
            elements::Paragraph::new("Header 1")
                .styled(style::Effect::Bold)
                .padded(1),
            None,
        )
        .cell(elements::Paragraph::new("Value 2").padded(1), None)
        .push()
        .expect("Invalid table row");
    table
        .row()
        .cell(
            elements::Paragraph::new("Header 2")
                .styled(style::Effect::Bold)
                .padded(1),
            None,
        )
        .cell(
            elements::Paragraph::new(
                "A long paragraph to demonstrate how wrapping works in tables.  Nice, right?",
            )
            .padded(1),
            None,
        )
        .push()
        .expect("Invalid table row");
    let list_layout = elements::LinearLayout::vertical()
        .element(elements::Paragraph::new(
            "Of course, you can use all other elements inside a table.",
        ))
        .element(
            elements::UnorderedList::new()
                .element(elements::Paragraph::new("Even lists!"))
                .element(
                    elements::Paragraph::new("And frames!")
                        .padded(genpdf::Margins::vh(0, 1))
                        .framed(style::LineStyle::new()),
                ),
        );
    table
        .row()
        .cell(
            elements::Paragraph::new("Header 3")
                .styled(style::Effect::Bold)
                .padded(1),
            None,
        )
        .cell(list_layout.padded(1), get_color(style::ColorName::GREY))
        .push()
        .expect("Invalid table row");
    doc.push(table);
    doc.push(elements::Break::new(1.5));

    doc.push(elements::Paragraph::new(
        "Now let’s print a long table to demonstrate how page wrapping works:",
    ));

    let mut table =
        elements::TableLayout::new(elements::ColumnWidths::PixelWidths(vec![50.0, 120.0]));
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true));

    table.register_header_row_callback_fn(|_| {
        let mut ht =
            elements::TableLayout::new(elements::ColumnWidths::PixelWidths(vec![50.0, 120.0]));
        ht.set_cell_decorator(elements::FrameCellDecorator::new(true, true));

        if let Some(color) = get_color(style::ColorName::GREY) {
            let mut hc1 = elements::Paragraph::new("Header Cell 1");
            hc1.set_bold(true);
            hc1.set_margins(2.into());
            let mut hc2 = elements::Paragraph::new("Header Cell 2");
            hc2.set_bold(true);
            hc2.set_margins(2.into());
            let hr = ht.row().cell(hc1, Some(color)).cell(hc2, Some(color));
            hr.push().expect("Invalid table row");
        }

        Ok(ht)
    });

    table.set_has_header_row_callback(true);

    let c2 = elements::Paragraph::new("Value");
    // if let Some(white) = get_color(style::ColorName::WHITE) {
    //     c2.set_color(white);
    // }

    table
        .row()
        .cell(
            elements::Paragraph::new("Index")
                .styled(style::Effect::Bold)
                .padded(1),
            None,
        )
        .cell(c2, None)
        .push()
        .expect("Invalid table row");
    for i in 0..10 {
        table
            .row()
            .cell(elements::Paragraph::new(format!("#{}", i)).padded(1), None)
            .cell(elements::Paragraph::new(LOREM_IPSUM).padded(1), None)
            .push()
            .expect("Invalid table row");
    }

    doc.push(table);

    doc.render_to_file(output_file)
        .expect("Failed to write output file");
    println!("PDF saved to  {}", output_file);
}

fn get_header_widget() -> elements::StyledElement<elements::LinearLayout> {
    let mut layout = elements::LinearLayout::vertical();
    layout.push(elements::Paragraph::new("Page #{page}").aligned(Alignment::Center));
    layout.push(elements::Break::new(1));
    layout.styled(style::Style::new().with_font_size(10))
}

// Only import the images if the feature is enabled. This helps verify our handling of feature toggles.
#[cfg(feature = "images")]
mod images {
    use super::*;

    const IMAGE_PATH_JPG: &'static str = "examples/images/test_image.jpg";

    pub fn do_image_test(doc: &mut genpdf::Document) {
        doc.push(elements::Paragraph::new(
            "Here is an example image with default position/scale:",
        ));
        doc.push(elements::Image::from_path(IMAGE_PATH_JPG).expect("Unable to load image"));
        doc.push(elements::Paragraph::new(
            "and here is one that is centered, rotated, and scaled some.",
        ));
        doc.push(
            elements::Image::from_path(IMAGE_PATH_JPG)
                .expect("Unable to load image")
                .with_alignment(Alignment::Center)
                .with_scale(genpdf::Scale::new(0.5, 2))
                .with_clockwise_rotation(45.0),
        );
        doc.push(elements::Paragraph::new(
            "For a full example of image functionality, please see images.pdf.",
        ));
        doc.push(elements::Break::new(1.5));
    }
}
