//
// Copyright (C) 2026 BayLibre
// Copyright (C) 2023 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Boot info from Android properties
//!
//! `get_boot_info()` is based on the one in
//! hardware/interfaces/security/keymint/aidl/default/hal/lib.rs.

use kmr_hal::env::{extract_patchlevel, get_property, OS_PATCHLEVEL_PROPERTY};
use kmr_wire::SetBootInfoRequest;
use log::error;

/// Get the boot information for the TA from the properties set by the
/// bootloader, as AOSP's non-secure reference HAL does.
///
/// These properties can be changed by Android: this is an interim solution
/// until the Root of Trust and the boot patch level are latched by the
/// bootloader in OP-TEE and read by the TA.
pub fn get_boot_info() -> SetBootInfoRequest {
    let vbmeta_digest = get_property("ro.boot.vbmeta.digest").unwrap_or_else(|_| "00".repeat(32));
    let verified_boot_hash = hex::decode(&vbmeta_digest).unwrap_or_else(|e| {
        error!("Failed to parse VBMeta digest '{vbmeta_digest}': {e:?}");
        vec![0; 32]
    });

    let device_boot_locked = match get_property("ro.boot.vbmeta.device_state").as_deref() {
        Ok("locked") => true,
        Ok("unlocked") => false,
        v => {
            error!("Unknown device_state {v:?}, treating as unlocked");
            false
        }
    };

    // The verified boot key must be all zeroes when the device is unlocked.
    let verified_boot_key = if device_boot_locked {
        let digest =
            get_property("ro.boot.vbmeta.public_key_digest").unwrap_or_else(|_| "00".repeat(32));
        hex::decode(&digest).unwrap_or_else(|e| {
            error!("Failed to parse verified boot key digest '{digest}': {e:?}");
            vec![0; 32]
        })
    } else {
        vec![0; 32]
    };

    // KeyMint VerifiedBootState: 0 Verified, 1 SelfSigned, 2 Unverified, 3 Failed
    let verified_boot_state = match get_property("ro.boot.verifiedbootstate").as_deref() {
        Ok("green") => 0,
        Ok("yellow") => 1,
        Ok("orange") => 2,
        Ok("red") => 3,
        v => {
            error!("Unknown verified boot state {v:?}, treating as Unverified");
            2
        }
    };

    // Fall back to the OS patch level if the boot patch level is not available.
    let boot_patchlevel_prop = get_property("ro.vendor.boot_security_patch").unwrap_or_else(|e| {
        error!("Failed to read the boot patch level: {e}");
        get_property(OS_PATCHLEVEL_PROPERTY).unwrap_or_else(|_| "1970-09-19".to_string())
    });
    let boot_patchlevel = extract_patchlevel(&boot_patchlevel_prop).unwrap_or(19700919);

    SetBootInfoRequest {
        verified_boot_key,
        device_boot_locked,
        verified_boot_state,
        verified_boot_hash,
        boot_patchlevel,
    }
}
