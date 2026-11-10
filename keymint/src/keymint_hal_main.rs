//
// Copyright (C) 2022 The Android Open-Source Project
// Copyright (C) 2026 BayLibre
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module implements the HAL service for KeyMint (Rust) with OP-TEE.
use kmr_hal::{
    register_binder_services, send_hal_info, HalServiceError, SerializedChannel, ALL_HALS,
};
use log::{debug, error, info};
use std::{
    ffi::CString,
    ops::DerefMut,
    panic,
    sync::{Arc, Mutex},
};

/// UUID of the KeyMint Rust TA.
const KEYMINT_TA_UUID: &str = "dba51a17-0563-11e7-93b1-6fa7b0071a51";

static SERVICE_INSTANCE: &str = "default";

#[derive(Debug)]
struct OpteeChannel(tee_optee_channel::OpteeChannel);

impl SerializedChannel for OpteeChannel {
    const MAX_SIZE: usize = 4000;
    fn execute(&mut self, serialized_req: &[u8]) -> binder::Result<Vec<u8>> {
        debug!("OpteeChannel::execute: {} bytes request", serialized_req.len());
        let rsp = self.0.execute(serialized_req).map_err(|e| {
            error!("OpteeChannel::execute failed: {e}");
            binder::Status::new_exception(
                binder::ExceptionCode::TRANSACTION_FAILED,
                Some(
                    &CString::new(format!(
                        "Failed to execute the request via the OP-TEE channel because of {e:?}"
                    ))
                    .unwrap(),
                ),
            )
        })?;
        debug!("OpteeChannel::execute: {} bytes response", rsp.len());
        Ok(rsp)
    }
}

/// Debug level on debuggable (eng, userdebug) builds, info level otherwise.
fn log_level() -> log::LevelFilter {
    match rustutils::android::system_properties::read_bool("ro.debuggable", false) {
        Ok(true) => log::LevelFilter::Debug,
        _ => log::LevelFilter::Info,
    }
}

fn main() {
    if let Err(HalServiceError(e)) = inner_main() {
        panic!("HAL service failed: {e:?}");
    }
}

fn inner_main() -> Result<(), HalServiceError> {
    // Initialize Android logging.
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("keymint-hal-optee-rust")
            .with_max_level(log_level())
            .with_log_buffer(android_logger::LogId::System),
    );
    // Redirect panic messages to logcat.
    panic::set_hook(Box::new(|panic_info| {
        error!("{panic_info}");
    }));

    info!("OP-TEE KeyMint Rust HAL service is starting.");

    info!("Starting thread pool now.");
    binder::ProcessState::start_thread_pool();

    // Create connection to the TA
    let connection = tee_optee_channel::OpteeChannel::new(KEYMINT_TA_UUID).map_err(|e| {
        HalServiceError(format!(
            "Failed to connect to the OP-TEE KeyMint TA {KEYMINT_TA_UUID} because of {e:?}."
        ))
    })?;
    let optee_channel = Arc::new(Mutex::new(OpteeChannel(connection)));

    register_binder_services(&optee_channel, ALL_HALS, SERVICE_INSTANCE)?;

    // Send the HAL service information to the TA
    send_hal_info(optee_channel.lock().unwrap().deref_mut())
        .map_err(|e| format!("Failed to populate HAL info: {e:?}"))?;
    debug!("HAL info sent to the TA");

    info!("Joining thread pool now.");
    binder::ProcessState::join_thread_pool();
    info!("KeyMint HAL service is terminating."); // should not reach here
    Ok(())
}
