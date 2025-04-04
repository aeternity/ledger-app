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

mod utils;
mod app_ui {
    pub mod address;
    pub mod menu;
    pub mod sign_data;
    pub mod sign_msg;
    pub mod sign_tx;
}
mod handlers {
    pub mod get_address;
    pub mod get_version;
    pub mod sign_data;
    pub mod sign_msg;
    pub mod sign_tx;
}

use app_ui::menu::ui_menu_main;
use handlers::{
    get_address::handler_get_address,
    get_version::handler_get_version,
    sign_data::handler_sign_data,
    sign_msg::handler_sign_message,
    sign_tx::{handler_sign_tx, TxContext},
};
use ledger_device_sdk::io::{ApduHeader, Comm, Reply, StatusWords};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

// Required for using String, Vec, format!...
extern crate alloc;

use ledger_device_sdk::nbgl::{init_comm, NbglReviewStatus, StatusType};

// P1 for GetAddress
const P1_CONFIRM_NOT_NEEDED: u8 = 0x00;
const P1_CONFIRM_NEEDED: u8 = 0x01;
// P1 for SignTx
const P1_SIGN_TX_START: u8 = 0x00;
const P1_SIGN_TX_MORE: u8 = 0x80;

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
    GetAddressParsingFail = 0xB200,
    DataWrongLength = 0xB300,
    DataSignFail = 0xB301,
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
    SignData,
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
            (2, P1_CONFIRM_NOT_NEEDED | P1_CONFIRM_NEEDED, 0) => Ok(Instruction::GetAddress {
                confirm_needed: value.p1 == P1_CONFIRM_NEEDED,
            }),
            (4, P1_SIGN_TX_START | P1_SIGN_TX_MORE, 0) => Ok(Instruction::SignTx {
                first_chunk: value.p1 == P1_SIGN_TX_START,
            }),
            (6, 0, 0) => Ok(Instruction::GetVersion),
            (8, 0, 0) => Ok(Instruction::SignMsg),
            (10, 0, 0) => Ok(Instruction::SignData),
            (2 | 4 | 6, _, _) => Err(AppSW::WrongP1P2),
            (_, _, _) => Err(AppSW::InsNotSupported),
        }
    }
}

fn show_status_and_home_if_needed(ins: &Instruction, tx_ctx: &mut TxContext, status: &AppSW) {
    let (show_status, status_type) = match (ins, status) {
        (
            Instruction::GetAddress {
                confirm_needed: true,
            },
            AppSW::Deny | AppSW::Ok,
        ) => (true, StatusType::Address),
        (Instruction::SignData, AppSW::Deny | AppSW::Ok) => {
            // TODO: should I change the StatusType?
            (true, StatusType::Message)
        }
        (Instruction::SignMsg, AppSW::Deny | AppSW::Ok) => (true, StatusType::Message),
        (Instruction::SignTx { .. }, AppSW::Deny | AppSW::Ok) if tx_ctx.is_finished() => {
            (true, StatusType::Transaction)
        }
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

    // Initialize reference to Comm instance for NBGL
    // API calls.
    init_comm(&mut comm);
    tx_ctx.home = ui_menu_main(&mut comm);
    tx_ctx.home.show_and_return();

    loop {
        let ins: Instruction = comm.next_command();

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
        show_status_and_home_if_needed(&ins, &mut tx_ctx, &_status);
    }
}

fn handle_apdu(comm: &mut Comm, ins: &Instruction, ctx: &mut TxContext) -> Result<(), AppSW> {
    match ins {
        Instruction::SignTx { first_chunk } => handler_sign_tx(comm, *first_chunk, ctx),
        Instruction::GetAddress { confirm_needed } => handler_get_address(comm, *confirm_needed),
        Instruction::GetVersion => handler_get_version(comm),
        Instruction::SignMsg => handler_sign_message(comm),
        Instruction::SignData => handler_sign_data(comm),
    }
}
