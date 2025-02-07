/*****************************************************************************
 *   Ledger App Boilerplate Rust.
 *   (c) 2023 Ledger SAS.
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 *****************************************************************************/
use crate::handlers::sign_tx::TxFirstChunk;
use crate::AppSW;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::{
    bitmaps::{CROSSMARK, EYE, VALIDATE_14},
    gadgets::{Field, MultiFieldReview},
};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use crate::settings::Settings;
#[cfg(any(target_os = "stax", target_os = "flex"))]
use include_gif::include_gif;
#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReview};

use alloc::format;

/// Displays a transaction and returns true if user approved it.
///
/// This method can return [`AppSW::TxDisplayFail`] error if the coin name length is too long.
///
/// # Arguments
///
/// * `tx` - Transaction to be displayed for validation
pub fn ui_display_tx(tx: &TxFirstChunk) -> Result<bool, AppSW> {
    use num_traits::cast::ToPrimitive;

    let amount_str = format!("{}", tx.amount.to_f64().unwrap());
    let fee_str = format!("{}", tx.fee.to_f64().unwrap());
    let to_str = format!("{}", tx.recipient);

    // Define transaction review fields
    let my_fields = [
        Field {
            name: "Amount",
            value: amount_str.as_str(),
        },
        Field {
            name: "Fee",
            value: fee_str.as_str(),
        },
        Field {
            name: "Destination",
            value: to_str.as_str(),
        },
        Field {
            name: "Payload",
            value: tx.payload.as_str(),
        },
    ];

    // Create transaction review
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let my_review = MultiFieldReview::new(
            &my_fields,
            &["Review ", "Transaction"],
            Some(&EYE),
            "Approve",
            Some(&VALIDATE_14),
            "Reject",
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
                "Review transaction\nto send AE",
                "",
                "Sign transaction\nto send AE",
            )
            .glyph(&FERRIS);

        Ok(review.show(&my_fields))
    }
}
