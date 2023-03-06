use std::iter::FromIterator;

use genpdf::elements::{Paragraph, TableLayout, UnorderedList};
use genpdf::error::{Error, ErrorKind};
use genpdf::fonts::{from_files, FontData, FontFamily};
use genpdf::style::{self, get_color, LineStyle, ORANGE};
use genpdf::utils::log;
use genpdf::{Borders, CustomPageDecorator, Document, Margins};

fn main() -> Result<(), Error> {
    let font_dir = "/Users/bharath/Work/Fonts/".to_string();
    let font = "OpenSans".to_string();
    // let japanese_font = "NotoSansCJKjp.otf".to_string();
    let font = get_pdf_font(font_dir.clone(), font)?;

    // let chinese_font = get_pdf_font(font_dir, japanese_font)?;

    let mut doc = Document::new(font);
    // doc.add_font_family(chinese_font);

    let output_file = "footer.pdf";

    // doc.push(footer_table);

    let mut d = CustomPageDecorator::new();

    let borders = Borders {
        top: Some(LineStyle::default().with_thickness(2.5).with_color(ORANGE)),
        right: None,
        bottom: None,
        left: Some(LineStyle::default()),
    };

    d.set_borders(Some(borders));
    d.set_margins(Some(Margins::trbl(1.0, 5.0, 5.0, 5.0)));
    d.register_footer_callback_fn(|_| {
        let mut footer_table = TableLayout::new_with_borders(
            genpdf::elements::ColumnWidths::PixelWidths(vec![90.0, 90.0]),
            true,
            true,
        );

        for i in 0..5 {
            let mut p = Paragraph::new(format!("Footer Row {} Col 1", i + 1));
            p.set_bold(true);
            p.set_alignment(genpdf::Alignment::Center);

            let mut p2 = Paragraph::new(format!("Footer Row {} Col 2", i + 1));
            p2.set_bold(true);
            p2.set_alignment(genpdf::Alignment::Center);
            footer_table
                .row()
                .cell(p, get_color(genpdf::style::ColorName::GREY))
                .cell(p2, get_color(genpdf::style::ColorName::GREY))
                .push()?;
        }
        // footer_table.set_margins(Margins::trbl(2.0, 0.0, 0.0, 0.0));
        Ok(footer_table)
    });
    d.register_header_callback_fn(|_| {
        let mut footer_table = TableLayout::new_with_borders(
            genpdf::elements::ColumnWidths::PixelWidths(vec![90.0, 90.0]),
            true,
            true,
        );

        for i in 0..3 {
            let mut p = Paragraph::new(format!("Header Row {} Col 1", i + 1));
            p.set_bold(true);
            p.set_alignment(genpdf::Alignment::Center);

            let mut p2 = Paragraph::new(format!("Header Row {} Col 2", i + 1));
            p2.set_bold(true);
            p2.set_alignment(genpdf::Alignment::Center);
            footer_table
                .row()
                .cell(p, get_color(genpdf::style::ColorName::GREY))
                .cell(p2, get_color(genpdf::style::ColorName::GREY))
                .push()?;
        }
        footer_table.set_margins(Margins::trbl(2.0, 0.0, 2.0, 0.0));
        // footer_table.set_margins(Margins::trbl(2.0, 0.0, 0.0, 0.0));
        Ok(footer_table)
    });
    doc.set_page_decorator(d);

    doc.push(genpdf::elements::Break::new(2));
    // create variable with long text
    let lorem_ipsum = "Underline ";
    let mut p = Paragraph::new("");
    p.set_underline(true);
    p.push(lorem_ipsum);
    let mut style = style::Style::new();
    style.set_underline(false);
    // style.set_font_size(35);
    // style.set_italic(true);
    p.push_styled(" NoUnderline", style);
    p.push(" NewUnderline");
    // p.push_styled("                         ", style);
    // p.push(" After blank");
    // p.set_font_size(20);
    p.set_bold(true);
    // p.set_underline(true);
    p.set_alignment(genpdf::Alignment::Center);
    doc.push(p);

    // #[cfg(feature = "images")]
    // let img = elements::Image::new("examples/images/cover.jpg");
    // doc.push(img);

    let desc = "The employee agrees to work on any public holiday that would otherwise be a working day for them if required. The employee also agrees not to work on any public holiday unless asked to do so. Select one: The employee will be paid reasonable compensation of  for being available to work on public holidays.The employee’s salary includes compensation for being available to work on public holidays. If the employee doesn’t work on a public holiday, they will get a paid day off if a public holiday falls on a day that would otherwise be a working day for them. If the employee works on a public holiday: - They will be paid their relevant daily pay or average daily pay, plus half that amount again for each hour worked (time and a half). - They will also get a paid day off at a later date unless the employee only ever works for the employer on public holidays. The date of this alternative holiday will be agreed between employer and employee. If they cannot agree, the employer can decide and give the employee at least 14 days’ notice.";

    let mut desc_para = Paragraph::new(desc);
    desc_para.set_font_size(10);
    doc.push(desc_para);

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
    ordered_list.push(Paragraph::new("ordered first"));

    let sub_list1 = UnorderedList::from_iter(vec![
        Paragraph::new("sub list 1"),
        Paragraph::new("sub list 2"),
        Paragraph::new("sub list 3"),
    ]);
    ordered_list.push_list(sub_list1);

    ordered_list.push(Paragraph::new("ordered second"));
    let sub_list2 = UnorderedList::from_iter(vec![
        Paragraph::new("sub list 4"),
        Paragraph::new("sub list 5"),
        Paragraph::new("sub list 6"),
    ]);
    ordered_list.push_list(sub_list2);
    unordered_list.push_list(ordered_list);

    doc.push(unordered_list);

    // data table
    let mut data_table = TableLayout::new_with_borders(
        genpdf::elements::ColumnWidths::PixelWidths(vec![90.0, 90.0]),
        true,
        true,
    );
    data_table.set_margins(Margins::trbl(2.0, 0.0, 2.0, 0.0));

    for i in 0..30 {
        let mut p = Paragraph::new(format!("Data Row {} Col 1", i + 1));
        p.set_bold(true);
        p.set_alignment(genpdf::Alignment::Center);

        let mut p2 = Paragraph::new(format!("Data Row {} Col 2", i + 1));
        p2.set_bold(true);
        p2.set_alignment(genpdf::Alignment::Center);
        data_table
            .row()
            .cell(p, get_color(genpdf::style::ColorName::CYAN))
            .cell(p2, get_color(genpdf::style::ColorName::PURPLE))
            .push()?;
    }
    doc.push(data_table);

    match doc.render_to_file(output_file) {
        Ok(_) => {}
        Err(e) => {
            log("Error while rendering doc to file", &format!("{e}"));
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
