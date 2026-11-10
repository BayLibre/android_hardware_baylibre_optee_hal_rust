/*
 * Copyright (C) 2023 The Android Open Source Project
 * Copyright (C) 2026 BayLibre
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! This module implements the HAL service for Gatekeeper (Rust) with OP-TEE.

use gk_hal::channel::SerializedChannel;
use log::{debug, error, info};
use std::ffi::CString;
use std::sync::{Arc, Mutex};

/// UUID of the Gatekeeper Rust TA.
const GATEKEEPER_TA_UUID: &str = "4d573443-6a56-4272-ac6f-2425af9ef9bb";

/// Name of `IGatekeeper` binder device instance.
static GK_INSTANCE: &str = "default";

static GK_SERVICE: &str = "android.hardware.gatekeeper.IGatekeeper";

/// Local error type for failures in the HAL service.
#[derive(Debug, Clone)]
struct HalServiceError(String);

impl From<String> for HalServiceError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug)]
struct OpteeChannel(tee_optee_channel::OpteeChannel);

impl SerializedChannel for OpteeChannel {
    /// Must match `GK_MAX_BUFFER_LENGTH` in the TA.
    const MAX_SIZE: usize = 8000;

    fn execute(&mut self, serialized_req: &[u8]) -> binder::Result<Vec<u8>> {
        debug!("OpteeChannel::execute: {} bytes request", serialized_req.len());
        let rsp = self.0.execute_single(serialized_req).map_err(|e| {
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
        panic!("HAL service failed: {:?}", e);
    }
}

fn inner_main() -> Result<(), HalServiceError> {
    // Initialize Android logging.
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("gatekeeper-hal-optee-rust")
            .with_max_level(log_level())
            .with_log_buffer(android_logger::LogId::System),
    );
    // Redirect panic messages to logcat.
    std::panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info);
    }));

    info!("OP-TEE Gatekeeper Rust HAL service is starting.");

    info!("Starting thread pool");
    binder::ProcessState::start_thread_pool();

    // Create connection to the TA.
    let connection = tee_optee_channel::OpteeChannel::new(GATEKEEPER_TA_UUID).map_err(|e| {
        HalServiceError(format!(
            "Failed to connect to the OP-TEE Gatekeeper TA {GATEKEEPER_TA_UUID} because of {e:?}."
        ))
    })?;
    let channel = Arc::new(Mutex::new(OpteeChannel(connection)));

    let gk_service = gk_hal::GatekeeperService::new_as_binder(channel);
    let service_name = format!("{GK_SERVICE}/{GK_INSTANCE}");
    binder::add_service(&service_name, gk_service.as_binder()).map_err(|e| {
        HalServiceError(format!("Failed to register service {service_name}: {e:?}"))
    })?;

    info!("Successfully registered Gatekeeper HAL service");
    binder::ProcessState::join_thread_pool();
    info!("Gatekeeper HAL service is terminating"); // should not reach here
    Ok(())
}
