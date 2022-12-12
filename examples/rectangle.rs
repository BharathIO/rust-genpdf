use std::fs::{self, File};
use std::io::BufWriter;

use printpdf::*;

fn mm_x(val: f64) -> printpdf::Mm {
    printpdf::Mm(val)
}
fn mm_y(val: f64) -> printpdf::Mm {
    printpdf::Mm(val)
}
fn main() {
    println!("Generating shapes..");
    let (doc, page1, layer1) =
        PdfDocument::new("printpdf graphics test", Mm(500.0), Mm(800.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let text = "Hello world";
    let data = fs::read("/Users/bharath/Work/Fonts/Arial-Regular.ttf").unwrap();

    let font = doc.add_external_font(&*data).unwrap();
    let black_color = printpdf::Color::Rgb(printpdf::Rgb::new(0.0, 0.0, 0.0, None));
    let red_color = printpdf::Color::Rgb(printpdf::Rgb::new(255.0, 0.0, 0.0, None));

    let grey_color = Color::Greyscale(Greyscale::new(0.6, None));
    // let out_line_color = Color::Greyscale(Greyscale::new(0.2, None));
    let fill_color = Color::Cmyk(Cmyk::new(0.0, 0.23, 0.0, 0.0, None));
    let _outline_color = Color::Rgb(Rgb::new(0.75, 1.0, 0.64, None));

    let line_points = vec![
        (Point::new(mm_x(50.0), mm_y(730.0)), false), //
        (Point::new(mm_x(50.0), mm_y(710.0)), false),
        (Point::new(mm_x(130.0), mm_y(710.0)), false),
        (Point::new(mm_x(130.0), mm_y(730.0)), false),
    ];

    let line1 = Line {
        points: line_points,
        is_closed: true,
        has_fill: true,
        has_stroke: true,
        is_clipping_path: false,
    };

    current_layer.set_fill_color(fill_color.clone());
    current_layer.set_outline_color(red_color.clone());
    current_layer.set_outline_thickness(2.0);

    // Draw rectangle shape
    current_layer.add_shape(line1);

    // `use_text` is a wrapper around making a simple string
    current_layer.set_fill_color(black_color.clone());
    current_layer.use_text(text, 35.0, Mm(60.0), Mm(715.0), &font);

    let next_cell = 130.0;
    let line_points2 = vec![
        (Point::new(mm_x(next_cell), mm_y(730.0)), false), //
        (Point::new(mm_x(next_cell), mm_y(710.0)), false),
        (Point::new(mm_x(70.0 + next_cell), mm_y(710.0)), false),
        (Point::new(mm_x(70.0 + next_cell), mm_y(730.0)), false),
    ];

    let line2 = Line {
        points: line_points2,
        is_closed: true,
        has_fill: true,
        has_stroke: true,
        is_clipping_path: false,
    };

    current_layer.set_fill_color(grey_color.clone());
    current_layer.set_outline_color(black_color.clone());
    current_layer.set_outline_thickness(2.0);

    // Draw rectangle shape
    current_layer.add_shape(line2);

    // `use_text` is a wrapper around making a simple string
    current_layer.set_fill_color(black_color.clone());
    current_layer.use_text("Column 2", 35.0, Mm(next_cell + 10.0), Mm(715.0), &font);

    doc.save(&mut BufWriter::new(File::create("rectangle.pdf").unwrap()))
        .unwrap();
    println!("File saved to rectangle.pdf");
}
