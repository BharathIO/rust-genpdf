use std::fs::File;
use std::io::BufWriter;

use printpdf::*;

fn main() {
    println!("Generating shapes..");
    let (doc, page1, layer1) =
        PdfDocument::new("printpdf graphics test", Mm(500.0), Mm(800.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    //     decorateCell, top_points: [Position { x: Mm(0.0), y: Mm(0.05) }, Position { x: Mm(200.0), y: Mm(0.05) }]
    // decorateCell, right_points: [Position { x: Mm(199.95), y: Mm(0.0) }, Position { x: Mm(199.95), y: Mm(5.067922988281249) }]
    // decorateCell, bottom_points: [Position { x: Mm(0.0), y: Mm(5.017922988281249) }, Position { x: Mm(200.0), y: Mm(5.017922988281249) }]
    // decorateCell, left_points: [Position { x: Mm(0.05), y: Mm(0.0) }, Position { x: Mm(0.05), y: Mm(5.067922988281249) }]

    // let p1 = Point::new(Mm(0.0), Mm(0.0));
    // let p2 = Point::new(Mm(200.0), Mm(0.05));
    // let p3 = Point::new(Mm(199.95), Mm(0.0));
    // let p4 = Point::new(Mm(199.95), Mm(5.067922988281249));
    // let p5 = Point::new(Mm(0.0), Mm(5.017922988281249));
    // let p6 = Point::new(Mm(200.0), Mm(5.017922988281249));
    // let p7 = Point::new(Mm(0.05), Mm(0.0));
    // let p8 = Point::new(Mm(0.05), Mm(5.067922988281249));

    // // Draw a rectangle
    // let p1 = Point::new(Mm(20.0), Mm(350.0)); // below
    // let p2 = Point::new(Mm(20.0), Mm(350.0)); // above top
    // let p3 = Point::new(Mm(100.0), Mm(50.0));
    // let p4 = Point::new(Mm(100.0), Mm(20.0));
    // let points1 = vec![(p1, false), (p2, false), (p3, false), (p4, false)];

    let bottom_x_start = Mm(15.0);
    let bottom_x_end = Mm(500.0);
    let height = Mm(400.0);
    // prepare 4 points printpdf rectangle shape
    // let p1 = Point::new(bottom_x_start, height);
    // let p2 = Point::new(bottom_x_start, height);
    // let p3 = Point::new(bottom_x_end, height);
    // let p4 = Point::new(bottom_x_end, height);
    // let line_points = vec![(p1, false), (p2, false), (p3, false), (p4, false)];

    // let top_left = Point::new(bottom_x_start, height);
    // let top_right = ;
    // let bottom_right;
    // let bottom_left;

    let p1_x = bottom_x_start;
    let p1_y = height;

    let p2_x = bottom_x_start;
    let p2_y = bottom_x_end;

    let p3_x = bottom_x_end;
    let p3_y = bottom_x_end;

    let p4_x = bottom_x_end;
    let p4_y = height;

    let line_points = vec![
        (Point::new(p1_x, p1_y), false),
        (Point::new(p2_x, p2_y), false),
        (Point::new(p3_x, p3_y), false),
        (Point::new(p4_x, p4_y), false),
    ];
    // Is the shape stroked? Is the shape closed? Is the shape filled?
    let line1 = Line {
        points: line_points,
        is_closed: false,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: true,
    };

    // let fill_color = printpdf::Color::Cmyk(printpdf::Cmyk::new(0.0, 0.23, 0.0, 0.0, None));
    let fill_color = printpdf::Color::Rgb(printpdf::Rgb::new(255.0, 0.0, 0.0, None));

    current_layer.set_fill_color(fill_color.clone());
    current_layer.set_outline_color(fill_color);

    // Draw first line
    current_layer.add_shape(line1);

    doc.save(&mut BufWriter::new(
        File::create("test_graphics.pdf").unwrap(),
    ))
    .unwrap();
    println!("File saved to test_graphics.pdf");
}
