use alloc::{borrow::ToOwned, vec::Vec, string::String};

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::{
    bitmaps::{CERTIFICATE, CROSSMARK, VALIDATE_14},
    gadgets::{Field, MultiFieldReview},
};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use include_gif::include_gif;
#[cfg(any(target_os = "stax", target_os = "flex"))]
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

    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let my_review = MultiFieldReview::new(
            &my_fields,
            &["Sign", "data"],
            Some(&CERTIFICATE),
            "Sign",
            Some(&VALIDATE_14),
            "Cancel",
            Some(&CROSSMARK),
        );

        Ok(my_review.show())
    }

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    {
        const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("icons/ae_64.gif", NBGL));
        // Create NBGL review. Maximum number of fields and string buffer length can be customised
        // with constant generic parameters of NbglReview. Default values are 32 and 1024 respectively.
        let review: NbglReview = NbglReview::new()
            .titles("Review data", "", "Sign data")
            .glyph(&FERRIS);

        Ok(review.show(&my_fields))
    }
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
