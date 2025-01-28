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
use crate::AppSW;
use core::str::FromStr;
use ledger_device_sdk::io;

pub fn handler_get_version(comm: &mut io::Comm) -> Result<(), AppSW> {
    if let Some((major, minor, patch)) = parse_version_string(env!("CARGO_PKG_VERSION")) {
        // In older versions of the app, the first byte of the get_version response was
        // used to store information about the app settings. This information turned out
        // later to be useless, and so it was removed from the app.
        // A 0x00 byte is appened before the version to maintain backward compatibility
        // with older versions of the app.
        comm.append(&[0x00, major, minor, patch]);
        Ok(())
    } else {
        Err(AppSW::VersionParsingFail)
    }
}

fn parse_version_string(input: &str) -> Option<(u8, u8, u8)> {
    // Split the input string by '.'.
    // Input should be of the form "major.minor.patch",
    // where "major", "minor", and "patch" are integers.
    let mut parts = input.split('.');
    let major = u8::from_str(parts.next()?).ok()?;
    let minor = u8::from_str(parts.next()?).ok()?;
    let patch = u8::from_str(parts.next()?).ok()?;
    Some((major, minor, patch))
}
