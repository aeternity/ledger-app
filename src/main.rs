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

#![no_std]
#![no_main]

// TODO: enable warnings later. this is done to make it easier to find error messages
#![allow(warnings)]

mod utils;
mod app_ui {
    pub mod address;
    pub mod menu;
    pub mod sign_msg;
    pub mod sign_tx;
}
mod handlers {
    pub mod get_address;
    pub mod get_version;
    pub mod sign_msg;
    pub mod sign_tx;
}

mod settings;

use app_ui::menu::ui_menu_main;
use handlers::{
    get_address::handler_get_address,
    get_version::handler_get_version,
    sign_msg::handler_sign_message,
    sign_tx::{handler_sign_tx, TxContext},
};
use ledger_device_sdk::io::{ApduHeader, Comm, Reply, StatusWords};
#[cfg(feature = "pending_review_screen")]
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::gadgets::display_pending_review;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::io::Event;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

// Required for using String, Vec, format!...
extern crate alloc;

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{init_comm, NbglReviewStatus, StatusType};

// P2 for last APDU to receive.
const P2_SIGN_TX_LAST: u8 = 0x00;
// P2 for more APDU to receive.
const P2_SIGN_TX_MORE: u8 = 0x80;
// P1 for first APDU number.
const P1_SIGN_TX_START: u8 = 0x00;
// P1 for maximum APDU number.
const P1_SIGN_TX_MAX: u8 = 0x03;

// Application status words.
#[repr(u16)]
#[derive(Clone, Copy, PartialEq)]
pub enum AppSW {
    Deny = 0x6985,
    WrongP1P2 = 0x6A86,
    InsNotSupported = 0x6D00,
    ClaNotSupported = 0x6E00,
    TxDisplayFail = 0xB001,
    AddrDisplayFail = 0xB002,
    TxWrongLength = 0xB004,
    TxParsingFail = 0xB005,
    TxHashFail = 0xB006,
    TxSignFail = 0xB008,
    KeyDeriveFail = 0xB009,
    VersionParsingFail = 0xB00A,
    MsgWrongLength = 0xB100,
    MsgHashFail = 0xB101,
    MsgSignFail = 0xB102,
    WrongApduLength = StatusWords::BadLen as u16,
    Ok = 0x9000,
}

impl From<AppSW> for Reply {
    fn from(sw: AppSW) -> Reply {
        Reply(sw as u16)
    }
}

/// Possible input commands received through APDUs.
pub enum Instruction {
    GetVersion,
    GetAddress { confirm_needed: bool },
    SignTx { first_chunk: bool },
    SignMsg,
}

impl TryFrom<ApduHeader> for Instruction {
    type Error = AppSW;

    /// APDU parsing logic.
    ///
    /// Parses INS, P1 and P2 bytes to build an [`Instruction`]. P1 and P2 are translated to
    /// strongly typed variables depending on the APDU instruction code. Invalid INS, P1 or P2
    /// values result in errors with a status word, which are automatically sent to the host by the
    /// SDK.
    ///
    /// This design allows a clear separation of the APDU parsing logic and commands handling.
    ///
    /// Note that CLA is not checked here. Instead the method [`Comm::set_expected_cla`] is used in
    /// [`sample_main`] to have this verification automatically performed by the SDK.
    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match (value.ins, value.p1, value.p2) {
            (2, 0 | 1, 0) => Ok(Instruction::GetAddress {
                confirm_needed: value.p1 != 0,
            }),
            (4, 0 | 0x80, 0) => Ok(Instruction::SignTx {
                first_chunk: value.p1 == 0
            }),
            (6, 0, 0) => Ok(Instruction::GetVersion),
            (8, 0, 0) => Ok(Instruction::SignMsg),
            (2 | 4 | 6, _, _) => Err(AppSW::WrongP1P2),
            (_, _, _) => Err(AppSW::InsNotSupported),
        }
    }
}

#[cfg(any(target_os = "stax", target_os = "flex"))]
fn show_status_and_home_if_needed(ins: &Instruction, tx_ctx: &mut TxContext, status: &AppSW) {
    let (show_status, status_type) = match (ins, status) {
        (_, _) => (false, StatusType::Transaction),
    };

    if show_status {
        let success = *status == AppSW::Ok;
        NbglReviewStatus::new()
            .status_type(status_type)
            .show(success);

        // call home.show_and_return() to show home and setting screen
        tx_ctx.home.show_and_return();
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    // Create the communication manager, and configure it to accept only APDU from the 0xe0 class.
    // If any APDU with a wrong class value is received, comm will respond automatically with
    // BadCla status word.
    let mut comm = Comm::new().set_expected_cla(0xe0);

    let mut tx_ctx = TxContext::new();

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    {
        // Initialize reference to Comm instance for NBGL
        // API calls.
        init_comm(&mut comm);
        tx_ctx.home = ui_menu_main(&mut comm);
        tx_ctx.home.show_and_return();
    }

    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    #[cfg(feature = "pending_review_screen")]
    display_pending_review(&mut comm);

    loop {
        #[cfg(any(target_os = "stax", target_os = "flex"))]
        let ins: Instruction = comm.next_command();

        #[cfg(not(any(target_os = "stax", target_os = "flex")))]
        let ins = if let Event::Command(ins) = ui_menu_main(&mut comm) {
            ins
        } else {
            continue;
        };

        let _status = match handle_apdu(&mut comm, &ins, &mut tx_ctx) {
            Ok(()) => {
                comm.reply_ok();
                AppSW::Ok
            }
            Err(sw) => {
                comm.reply(sw);
                sw
            }
        };
        #[cfg(any(target_os = "stax", target_os = "flex"))]
        show_status_and_home_if_needed(&ins, &mut tx_ctx, &_status);
    }
}

fn handle_apdu(comm: &mut Comm, ins: &Instruction, ctx: &mut TxContext) -> Result<(), AppSW> {
    match ins {
        Instruction::SignTx { first_chunk } => handler_sign_tx(comm, *first_chunk, ctx),
        Instruction::GetAddress { confirm_needed } => handler_get_address(comm, *confirm_needed),
        Instruction::GetVersion => handler_get_version(comm),
        Instruction::SignMsg => handler_sign_message(comm),
    }
}
