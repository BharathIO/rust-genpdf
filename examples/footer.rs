use genpdf::elements::{Paragraph, TableLayout};
use genpdf::error::{Error, ErrorKind};
use genpdf::fonts::{from_files, FontData, FontFamily};
use genpdf::style::get_color;
use genpdf::{CustomPageDecorator, Document, Margins};

fn main() -> Result<(), Error> {
    let font_dir = "/Users/bharath/Work/Fonts/".to_string();
    let font = "OpenSans".to_string();
    let font = get_pdf_font(font_dir, font)?;

    let mut doc = Document::new(font);
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
    doc.push(footer_table);

    doc.render_to_file(output_file)
        .expect("Failed to write output file");
    println!("PDF saved to  {}", output_file);
    Ok(())
}
pub fn get_pdf_font(font_dir: String, font: String) -> Result<FontFamily<FontData>, Error> {
    println!("Font dir: {}", &font_dir);
    match from_files(font_dir.clone(), &font, None) {
        Ok(f) => Ok(f),
        Err(e) => {
            let err = format!(
                "Error loading font {} from directory {}, Error: {}",
                font, font_dir, e
            );
            println!("{}", err);
            return Err(Error::new(err, ErrorKind::Internal));
        }
    }
}
