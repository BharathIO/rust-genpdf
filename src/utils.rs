// SPDX-FileCopyrightText: 2020-2021 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Low-level PDF rendering utilities.
//!
//! This module provides low-level abstractions over [`printpdf`][]:  A [`Renderer`][] creates a
//! document with one or more pages with different sizes.  A [`Page`][] has one or more layers, all
//! of the same size.  A [`Layer`][] can be used to access its [`Area`][].
//!
//! An [`Area`][] is a view on a full layer or on a part of a layer.  It can be used to print
//! lines and text.  For more advanced text formatting, you can create a [`TextSection`][] from an
//! [`Area`][].
//!
//! [`printpdf`]: https://docs.rs/printpdf/latest/printpdf
//! [`Renderer`]: struct.Renderer.html
//! [`Page`]: struct.Page.html
//! [`Layer`]: struct.Layer.html
//! [`Area`]: struct.Area.html
//! [`TextSection`]: struct.TextSection.html

use std::{env, path::PathBuf};

use crate::{
    error::Error,
    fonts::{from_files, FontData, FontFamily},
};

/// defualt font dir
///
pub fn get_default_font_dir() -> PathBuf {
    let mut font_dir = PathBuf::new();
    let dir = env::current_dir().ok();
    if let Some(dir) = dir {
        font_dir = dir.join("assets/fonts/Liberation/");
    }
    font_dir
}

const DEFAULT_FONT_NAME: &str = "LiberationSans";
/// get_default_font
///
pub fn get_default_font() -> Result<FontFamily<FontData>, Error> {
    get_font(DEFAULT_FONT_NAME)
}

/// get_font
///
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

    // let pixel_width = Px::from(10);
    // let pixel_height = Px::from(10);

    Ok(font)
}
