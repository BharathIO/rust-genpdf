use genpdf::elements::ColumnWidths;
use genpdf::elements::Paragraph;
use genpdf::elements::TableLayout;
use genpdf::error::Error;
use genpdf::error::ErrorKind;
use genpdf::fonts::from_files;
use genpdf::fonts::FontData;
use genpdf::fonts::FontFamily;
use genpdf::style;
use genpdf::style::get_color;
use genpdf::style::Color;
use genpdf::Document;
use genpdf::Margins;

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
fn main() -> Result<(), Error> {
    println!("Generating shapes..");

    let font_dir = "/Users/bharath/Work/Fonts/".to_string();
    let font = "OpenSans".to_string();
    let default_font = match get_pdf_font(font_dir, font) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    let blue_color = style::BLUE;
    let grey_color = style::GREY;
    let black_color = style::BLACK;
    let cyan_color = style::CYAN;
    let green_color = style::GREEN;

    let mut doc = Document::new(default_font);
    // doc.set_font_color(style::BLACK);
    // doc.set_font_size(10);

    let mut tbl = TableLayout::new(ColumnWidths::PixelWidths(vec![30.0, 30.0, 30.0]));
    // let mut tbl =
    //     TableLayout::new_with_borders(ColumnWidths::PixelWidths(vec![20.0, 40.0]), true, true);
    tbl.set_margins(Margins::trbl(20.0, 10.0, 10.0, 20.0));

    let mut tr = tbl.row();

    // let tr = table.createrow();
    // tr.createcell("Hello");
    // tr.createcell("World");
    // tr.createcell("Hello");
    // tr.finish();

    let mut c1 = Paragraph::new("H-1").aligned(genpdf::Alignment::Center);
    // c1.set_color(black_color);
    let mut c2 = Paragraph::new("H-2").aligned(genpdf::Alignment::Center);
    // c2.set_color(black_color);
    let mut c3 = Paragraph::new("H-3").aligned(genpdf::Alignment::Center);

    tr.cell(c1, Some(grey_color));
    tr.cell(c2, Some(grey_color));
    tr.cell(c3, Some(grey_color));

    match tr.push() {
        Ok(_) => {}
        Err(e) => return Err(e),
    };

    // data rows
    for i in 0..10 {
        let mut tr = tbl.row();
        let mut c1 = Paragraph::new(format!("Cell1-{}", i)).aligned(genpdf::Alignment::Center);
        // c1.set_color(black_color);
        let mut c2 = Paragraph::new(format!("Cell2-{}", i)).aligned(genpdf::Alignment::Center);
        // c2.set_color(black_color);
        let mut c3 = Paragraph::new(format!("Cell3-{}", i)).aligned(genpdf::Alignment::Center);
        c3.set_bold();
        if i == 9 {
            c3.set_italic();
            // set orange color
            c3.set_color(get_color("pink").unwrap());
        }
        // c3.set_color(green_color);
        // c3.set_color(get_color("white").unwrap());

        // if i % 2 == 0 {
        //     color = Some(cyan_color);
        // } else {
        //     color = Some(green_color);
        // }
        let color = match i {
            0 => get_color("red"),
            1 => get_color("blue"),
            2 => get_color("green"),
            3 => get_color("cyan"),
            4 => get_color("magenta"),
            5 => get_color("yellow"),
            6 => get_color("pink"),
            7 => get_color("white"),
            8 => get_color("grey"),
            _ => get_color("white"),
        };
        tr.cell(c1, color);
        tr.cell(c2, color);
        tr.cell(c3, color);

        match tr.push() {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
    }

    doc.push(tbl);
    doc.render_to_file("table.pdf")?;
    println!("Done!, Saved to table.pdf !!");

    Ok(())
}
