use genpdf::elements::{OrderedList, Paragraph};
use genpdf::error::{Error, ErrorKind};
use genpdf::fonts::{from_files, FontData, FontFamily};
use genpdf::style::{get_color, Style};
use genpdf::{CustomPageDecorator, Document, Margins};

fn main() -> Result<(), Error> {
    let font_dir = "/Users/bharath/Work/Fonts/".to_string();
    let font = "OpenSans".to_string();
    let font = get_pdf_font(font_dir.clone(), font)?;

    let mut doc = Document::new(font);

    let p_text = "On April 1, 303, __________________________________ in Sturgis, Michigan, seven men aged 17 to 20 placed ___________________________________ signs all over town that read \"All your base are belong to us. You have no chance to survive make your time.\" They said they were playing an April Fools joke by mimicking the famous Flash animation which ubiquitously depicted the slogan. Not many people who saw the signs were familiar with the joke, however. Many residents were upset that the signs appeared while the U.S. was at war with Iraq, and police chief Eugene Alli said the signs could be \"a borderline terrorist threat depending on what someone interprets it to mean.\" [1]".to_owned();
    let mut p = Paragraph::default();
    p.push(p_text);
    p.set_line_spacing(1.5);
    doc.push(p);

    let mut d = CustomPageDecorator::new();
    d.set_margins(Some(Margins::trbl(10, 20, 10, 30)));
    doc.set_page_decorator(d);
    let output_file = "contract.pdf";

    let lorem_ipsum = "CONTRACT OF ";
    let mut p = Paragraph::new("");
    p.set_underline(true);
    p.set_font_size(17);
    p.push(lorem_ipsum);
    // let mut style = style::Style::new();
    // style.set_underline(true);
    // style.set_italic();
    p.push(" EMPLOYMENT");
    p.set_bold(true);
    p.set_alignment(genpdf::Alignment::Center);
    // // doc.push(Break::new(3));
    p.set_margins(Margins::trbl(15, 0, 0, 0));
    // doc.push(p);
    // // doc.push(Break::new(2));

    let mut p2 = Paragraph::new("MADE AND ENTERED INTO BY AND BETWEEN:");
    p2.set_alignment(genpdf::Alignment::Center);
    p2.set_bold(true);
    p2.set_margins(Margins::trbl(7, 0, 0, 0));
    // doc.push(p2);

    let mut p3 = Paragraph::default();
    p3.set_margins(Margins::trbl(10, 0, 0, 0));
    for _ in 0..80 {
        p3.push("_");
    }
    // doc.push(p3);
    // doc.push(Paragraph::new("with address at:"));

    for _ in 0..2 {
        let mut p4 = Paragraph::default();
        // p4.set_margins(Margins::trbl(10, 0, 0, 0));
        for _ in 0..80 {
            p4.push("_");
        }
        // doc.push(p4);
    }

    let str = "WHEREBY THE PARTIES AGREE AS FOLLOWS:";
    let mut p5 = Paragraph::new(str);
    p5.set_bold(true);
    p5.set_margins(Margins::trbl(10, 0, 0, 0));
    // doc.push(p5);

    let mut bullet_style = Style::default();
    bullet_style.set_bold(true);
    bullet_style.set_color(get_color("RED".into()).unwrap());
    bullet_style.set_underline(true);

    let mut ol = OrderedList::new();
    ol.set_bullet_style(bullet_style);

    let mut ol_p1 = Paragraph::new("APPOINTMENT");
    ol_p1.set_bold(true);
    ol_p1.set_underline(true);
    // ol_p1.set_margins(Margins::trbl(0, 0, 0, 10));

    // let bottom_mr = Margins::trbl(0, 0, 10, 0);
    // ol_p1.push("The EMPLOYEE, who hereby accepts the appointment and is appointed as a ________________________________________________ for the EMPLOYER.");
    ol.push(ol_p1);

    let sub_text = "The EMPLOYEE, who hereby accepts the appointment and is appointed as a ________________________________________________ for the EMPLOYER.";
    let sub_para = Paragraph::new(sub_text);
    // sub_para.set_margins(Margins::trbl(2, 0, 5, 5));

    let sub_text2 = "AThis agreement will become affective as from ___________ (insert date) and it will continue for an indefinite period until it has been cancelled in terms hereof.";
    let mut sub_para2 = Paragraph::new(sub_text2);
    sub_para2.set_line_spacing(20.0);
    // sub_para2.set_margins(Margins::trbl(2, 0, 5, 5));

    // let mut ll = LinearLayout::vertical();
    // ll.push(sub_para);

    let mut ol_p2 = Paragraph::new("DURATION");
    ol_p2.set_bold(true);
    ol_p2.set_underline(true);
    // ol_p2.set_margins(Margins::trbl(5, 0, 0, 0));
    // ol_p1.push("The EMPLOYEE, who hereby accepts the appointment and is appointed as a ________________________________________________ for the EMPLOYER.");
    ol.push(ol_p2);

    let mut app_sub_list = OrderedList::new();
    app_sub_list.push(sub_para);
    app_sub_list.push(sub_para2);

    ol.push_list(app_sub_list);

    ol.set_margins(Margins::trbl(10, 0, 0, 0));
    // doc.push(ol);

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
