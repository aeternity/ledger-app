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
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use primitive_types::U256;

use crate::handlers::sign_tx::TxFirstChunk;
use crate::AppSW;

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReview};

/// Displays a transaction and returns true if user approved it.
///
/// This method can return [`AppSW::TxDisplayFail`] error if the coin name length is too long.
///
/// # Arguments
///
/// * `tx` - Transaction to be displayed for validation
pub fn ui_display_tx(tx: &TxFirstChunk) -> Result<bool, AppSW> {
    let amount_str = display_amount(tx.amount);
    let fee_str = display_amount(tx.fee);
    let to_str = tx.recipient.clone();

    // Define transaction review fields
    let mut my_fields = Vec::from([
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
    ]);

    if !tx.payload.is_empty() {
        my_fields.push(Field {
            name: "Payload",
            value: tx.payload.as_str(),
        });
    }

    // Create transaction review

    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("icons/ae_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("icons/ae_16x16.gif", NBGL));
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

/// Convert an amount in Aettos to an amount in AE.
///
/// Since there's no need to deal with floating-point numbers, the conversion
/// is done by converting the amount to String and moving the decimal point 18
/// places to left.
fn display_amount(amount: U256) -> String {
    const DECIMAL_PLACES: usize = 18;

    // Pad the amount in Aettos with 18 leading zeros
    let padded = ["0".repeat(DECIMAL_PLACES), amount.to_string()].concat();

    // Move the decimal point 18 places to the left (divide by 10^18)
    let (left, right) = padded.split_at(padded.len() - DECIMAL_PLACES);

    // Remove leading zeros from the decimal part
    let dec = left.chars().skip_while(|c| *c == '0').collect::<String>();
    // Remove trailing zeros from the fractional part
    let frac = right
        .chars()
        .rev()
        .skip_while(|c| *c == '0')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();

    let mut output = String::new();

    if dec.is_empty() {
        output.push('0');
    } else {
        output.push_str(&dec);
    }

    if !frac.is_empty() {
        output.push('.');
        output.push_str(&frac);
    }

    output
}
