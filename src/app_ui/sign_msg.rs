#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::{
    bitmaps::{CROSSMARK, CERTIFICATE, VALIDATE_14},
    gadgets::{Field, MultiFieldReview},
};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use include_gif::include_gif;
#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReview};

use crate::AppSW;

pub fn ui_display_msg(message_bytes: &[u8]) -> Result<bool, AppSW> {
    let message = core::str::from_utf8(message_bytes).unwrap();

    let my_fields = [
        Field {
            name: "Message",
            value: message,
        },
    ];

    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let my_review = MultiFieldReview::new(
            &my_fields,
            &["Sign", "message"],
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
            .titles(
                "Review message",
                "",
                "Sign message",
            )
            .glyph(&FERRIS);

        Ok(review.show(&my_fields))
    }
}
