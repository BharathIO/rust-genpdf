use std::iter::FromIterator;

use genpdf::elements::{Paragraph, TableLayout, UnorderedList};
use genpdf::error::{Error, ErrorKind};
use genpdf::fonts::{from_files, FontData, FontFamily};
use genpdf::style::{self, get_color};
use genpdf::{CustomPageDecorator, Document, Margins};

fn main() -> Result<(), Error> {
    let font_dir = "/Users/bharath/Work/Fonts/".to_string();
    let font = "OpenSans".to_string();
    // let japanese_font = "NotoSansCJKjp.otf".to_string();
    let font = get_pdf_font(font_dir.clone(), font)?;

    // let chinese_font = get_pdf_font(font_dir, japanese_font)?;

    let mut doc = Document::new(font);
    // doc.add_font_family(chinese_font);

    let mut d = CustomPageDecorator::new();
    d.set_margins(Some(Margins::all(10.0)));
    doc.set_page_decorator(d);
    let output_file = "footer.pdf";

    let mut footer_table = TableLayout::new_with_borders(
        genpdf::elements::ColumnWidths::PixelWidths(vec![95.0, 95.0]),
        true,
        true,
    );

    let mut p = Paragraph::new("1 Footer #{page}");
    p.set_bold();
    p.set_alignment(genpdf::Alignment::Center);

    let mut p2 = Paragraph::new("2 Footer #{page}");
    p2.set_bold();
    p2.set_alignment(genpdf::Alignment::Center);
    footer_table
        .row()
        .cell(p, get_color(genpdf::style::ColorName::GREY))
        .cell(p2, get_color(genpdf::style::ColorName::GREY))
        .push()?;
    footer_table.set_margins(Margins::trbl(2.0, 0.0, 0.0, 0.0));
    // doc.push(footer_table);

    doc.push(genpdf::elements::Break::new(2));
    // create variable with long text
    let lorem_ipsum = "CONTRACT OF ";
    let mut p = Paragraph::new("");
    p.set_underline();
    p.push(lorem_ipsum);
    let mut style = style::Style::new();
    style.set_underline();
    style.set_italic();
    p.push(" EMPLOYMENT");
    // p.push_styled("                         ", style);
    // p.push(" After blank");
    // p.set_font_size(20);
    p.set_bold();
    // p.set_underline();
    p.set_alignment(genpdf::Alignment::Center);
    doc.push(p);
    // let bp1 = BulletPoint::new(Paragraph::new("Bullet Point 1"));
    // let bp2 = BulletPoint::new(Paragraph::new("Bullet Point 2"));
    // doc.push(bp1);
    // doc.push(bp2);

    let mut unordered_list = genpdf::elements::UnorderedList::new();
    unordered_list.push(Paragraph::new("first"));
    unordered_list.push(Paragraph::new("second"));
    unordered_list.push(Paragraph::new("third"));

    doc.push(genpdf::elements::Break::new(2));

    let mut ordered_list = genpdf::elements::OrderedList::new();
    ordered_list.push(Paragraph::new("ordered first"), None);

    let sub_list1 = UnorderedList::from_iter(vec![
        Paragraph::new("sub list 1"),
        Paragraph::new("sub list 2"),
        Paragraph::new("sub list 3"),
    ]);
    ordered_list.push_list(sub_list1);

    ordered_list.push(Paragraph::new("ordered second"), None);
    let sub_list2 = UnorderedList::from_iter(vec![
        Paragraph::new("sub list 4"),
        Paragraph::new("sub list 5"),
        Paragraph::new("sub list 6"),
    ]);
    ordered_list.push_list(sub_list2);
    unordered_list.push_list(ordered_list);

    // doc.push(unordered_list);

    match doc.render_to_file(output_file) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            return Err(Error::new(e.to_string(), ErrorKind::Internal));
        }
    }
    // .expect("Failed to write output file");
    println!("PDF saved to  {}", output_file);
    Ok(())
}
pub fn get_pdf_font(font_dir: String, font: String) -> Result<FontFamily<FontData>, Error> {
    println!("Font dir: {}", &font_dir);
    match from_files(font_dir.clone(), &font, None) {
        Ok(f) => Ok(f),
        Err(e) => {
            let err = format!("{}", e);
            println!("{}", err);
            return Err(Error::new(err, ErrorKind::Internal));
        }
    }
}
