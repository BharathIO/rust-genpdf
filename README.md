<!---
Copyright (C) 2020 Robin Krahl <robin.krahl@ireas.org>
SPDX-License-Identifier: CC0-1.0
-->

# genpdf-rs

A user-friendly PDF generator written in pure Rust.

[Documentation](https://docs.rs/genpdf)

`genpdf` is a high-level PDF generator built on top of [`printpdf`][] and
[`rusttype`][].  It takes care of the page layout and text alignment and
renders a document tree into a PDF document.  All of its dependencies are
written in Rust, so you don’t need any pre-installed libraries or tools.

[`printpdf`]: https://lib.rs/crates/printpdf
[`rusttype`]: https://lib.rs/crates/rusttype

<!-- Keep in sync with src/lib.rs -->
```rust
// Load a font from the file system
let font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)
    .expect("Failed to load font family");
// Create a document and set the default font family
let mut doc = genpdf::Document::new(font_family);
// Change the default settings
doc.set_title("Demo document");
// Customize the pages
let mut decorator = genpdf::SimplePageDecorator::new();
decorator.set_margins(10);
doc.set_page_decorator(decorator);
// Add one or more elements
doc.push(genpdf::elements::Paragraph::new("This is a demo document."));
// Render the document and write it to a file
doc.render_to_file("output.pdf").expect("Failed to write PDF file");
```

For a complete example with all supported elements, see the
[`examples/demo.rs`][] file that generates [this PDF document][].

[`examples/demo.rs`]: https://git.sr.ht/~ireas/genpdf-rs/tree/master/examples/demo.rs
[this PDF document]: https://genpdf-rs.ireas.org/examples/demo.pdf

For more information, see the [API documentation](https://docs.rs/genpdf).

## Features

- PDF generation in pure Rust
- Text rendering with support for setting the font family, style and size as
  well as the text color and text effects (bold or italic) and with kerning
- Text wrapping at word boundaries and optional hyphenation
- Layout of elements sequentially or in tables
- Rudimentary support for shapes
- Page headers and custom page decorations
- Embedding images (scale, position, rotate).

## Cargo Features

This crate has the following Cargo features (deactivated per default):

- `images`: Adds support for embedding images using the [`image`][] crate.
- `hyphenation`:  Adds support for hyphenation using the [`hyphenation`][] crate.

[`hyphenation`]: https://lib.rs/crates/hyphenation
[`image`]: https://lib.rs/crates/image

## Roadmap

These features are currently not supported but planned for future versions:
- Improved support for drawing shapes
- Advanced text formatting

See also the [`genpdf-rs` issue tracker](https://todo.sr.ht/~ireas/genpdf-rs).

## Alternatives

- [`printpdf`][] is the low-level PDF library used by `genpdf`.  It provides
  more control over the generated document, but you have to take care of all
  details like calculating the width and height of the rendered text, arranging
  the elements and distributing them on multiple pages.
- [`latex`][] generates LaTeX documents from Rust.  It requires a LaTex
  installation to generate the PDF files.  Also, escaping user input is a
  non-trivial problem and not supported by the crate.
- [`tectonic`][] is a TeX engine based on XeTeX.  It is partly written in C and
  has some non-Rust dependencies.
- [`wkhtmltopdf`][] generates PDF documents from HTML using the `wkhtmltox`
  library.  It requires a pre-installed library and does not support custom
  elements.

[`latex`]: https://lib.rs/crates/latex
[`tectonic`]: https://lib.rs/crates/tectonic
[`wkhtmltopdf`]: https://lib.rs/crates/wkhtmltopdf

## Minimum Supported Rust Version

This crate supports at least Rust 1.45.0 or later.

## Contributing

Contributions to this project are welcome!  Please submit patches to the
mailing list [~ireas/public-inbox@lists.sr.ht][] ([archive][]) using the
`[PATCH genpdf-rs]` subject prefix.  For more information, see the
[Contributing Guide][].

[~ireas/public-inbox@lists.sr.ht]: mailto:~ireas/public-inbox@lists.sr.ht
[archive]: https://lists.sr.ht/~ireas/public-inbox
[Contributing Guide]: https://man.sr.ht/~ireas/guides/contributing.md

If you are looking for a good starting point, have a look at the [issues with
the label “good first issue”][issues] in `genpdf-rs`’s issue tracker.

[issues]: https://todo.sr.ht/~ireas/genpdf-rs?search=label:%22good%20first%20issue%22%20status%3Aopen

## Contact

For bug reports, feature requests and other messages, please send a mail to
[~ireas/public-inbox@lists.sr.ht][] ([archive][]) using the `[genpdf-rs]`
prefix in the subject.

## License

This project is dual-licensed under the [Apache-2.0][] and [MIT][] licenses.
The documentation and examples contained in this repository are licensed under
the [Creative Commons Zero][CC0] license.  You can find a copy of the license
texts in the `LICENSES` directory.

`genpdf-rs` complies with [version 3.0 of the REUSE specification][reuse].

[Apache-2.0]: https://opensource.org/licenses/Apache-2.0
[MIT]: https://opensource.org/licenses/MIT
[CC0]: https://creativecommons.org/publicdomain/zero/1.0/
[reuse]: https://reuse.software/practices/3.0/
