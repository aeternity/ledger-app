use alloc::{borrow::ToOwned, string::String, vec::Vec};

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReview};

use base64::prelude::{Engine, BASE64_STANDARD};

use crate::AppSW;

pub fn ui_display_data(data_bytes: &[u8]) -> Result<bool, AppSW> {
    let data = format_data(data_bytes);

    let my_fields = match &data {
        Some(string) => Vec::from([Field {
            name: "Data",
            value: string,
        }]),
        None => Vec::new(),
    };

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("icons/ae_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("icons/ae_16x16.gif", NBGL));
    // Create NBGL review. Maximum number of fields and string buffer length can be customised
    // with constant generic parameters of NbglReview. Default values are 32 and 1024 respectively.
    let review: NbglReview = NbglReview::new()
        .titles("Review data", "", "Sign data")
        .glyph(&FERRIS);

    Ok(review.show(&my_fields))
}

fn format_data(data: &[u8]) -> Option<String> {
    if data.len() > 50 {
        None
    } else {
        match core::str::from_utf8(data) {
            Ok(utf8_str) if utf8_str.is_ascii() => Some(utf8_str.to_owned()),
            _ => Some(BASE64_STANDARD.encode(data)),
        }
    }
}
