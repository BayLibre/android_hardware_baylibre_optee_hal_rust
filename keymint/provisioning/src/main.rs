//
// Copyright (C) 2026 BayLibre
// Copyright (C) 2025 The Android Open Source Project
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

//! Provision the KeyMint attestation IDs in the KeyMint TA, once per device.
//! The KeyMint TA refuses any later provisioning.
//!
//! The command line follows the Trusty KeyMint provisioning tool
//! (system/core/trusty/keymint/provisioning): each ID can be given as an
//! option, otherwise it is read from the product properties.
//!
//! Usage: keymint-optee-provision-ids set-attestation-ids [--dry-run] [OPTIONS]

use clap::{Args, Parser, Subcommand};
use kmr_wire::{AsCborValue, AttestationIdInfo, SetAttestationIdsRequest};
use rustutils::android::system_properties::read;
use std::process::ExitCode;
use tee_optee_bindings::TEEC_ERROR_ACCESS_CONFLICT;
use tee_optee_channel::OpteeChannel;

/// UUID of the KeyMint Rust TA
const KEYMINT_TA_UUID: &str = "dba51a17-0563-11e7-93b1-6fa7b0071a51";

/// KeyMint TA command, see vendor/baylibre/optee_ta_rust/security_ta/keymint/src/command.rs
const SET_ATTESTATION_IDS: u32 = 0x10001;

#[derive(Parser, Debug)]
#[clap(about = "KeyMint OP-TEE provisioning tool")]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug, Clone)]
enum Action {
    /// Provisions attestation IDs to the KeyMint TA.
    SetAttestationIds(AttestationIdsArgs),
}

#[derive(Args, Clone, Debug, Default)]
struct AttestationIdsArgs {
    /// Print the IDs without provisioning them.
    #[arg(long)]
    dry_run: bool,

    /// Set brand.
    #[arg(long, short = 'b')]
    brand: Option<String>,

    /// Set device.
    #[arg(long, short = 'd')]
    device: Option<String>,

    /// Set product.
    #[arg(long, short = 'p')]
    product: Option<String>,

    /// Set serial.
    #[arg(long, short = 's')]
    serial: Option<String>,

    /// Set manufacturer.
    #[arg(long, short = 'M')]
    manufacturer: Option<String>,

    /// Set model.
    #[arg(long, short = 'm')]
    model: Option<String>,

    /// Set MEID.
    #[arg(long)]
    meid: Option<String>,

    /// Set IMEI (slot 0).
    #[arg(long)]
    imei: Option<String>,

    /// Set second IMEI (slot 1).
    #[arg(long)]
    imei2: Option<String>,
}

fn property(name: &str) -> Result<String, String> {
    match read(name) {
        Ok(Some(value)) if !value.is_empty() => Ok(value),
        Ok(_) => Err(format!("{name} is not set")),
        Err(e) => Err(format!("cannot read {name}: {e:?}")),
    }
}

/// Same lookup as VTS: ro.product.<name>_for_attestation, then
/// ro.product.vendor.<name>, then ro.product.<name>.
fn product_property(name: &str) -> Result<String, String> {
    property(&format!("ro.product.{name}_for_attestation"))
        .or_else(|_| property(&format!("ro.product.vendor.{name}")))
        .or_else(|_| property(&format!("ro.product.{name}")))
}

/// The option if given, otherwise the property: a missing property is an
/// error, as the TA stores the IDs only once.
fn arg_or_prop(
    arg: &Option<String>,
    prop: impl FnOnce() -> Result<String, String>,
) -> Result<Vec<u8>, String> {
    match arg {
        Some(value) => Ok(value.as_bytes().to_vec()),
        None => prop().map(String::into_bytes),
    }
}

/// The option if given, otherwise empty.
fn arg_or_default(arg: &Option<String>) -> Vec<u8> {
    arg.as_ref().map(|value| value.as_bytes().to_vec()).unwrap_or_default()
}

fn collect_attestation_ids(args: &AttestationIdsArgs) -> Result<AttestationIdInfo, String> {
    Ok(AttestationIdInfo {
        brand: arg_or_prop(&args.brand, || product_property("brand"))?,
        device: arg_or_prop(&args.device, || product_property("device"))?,
        product: arg_or_prop(&args.product, || product_property("name"))?,
        serial: arg_or_prop(&args.serial, || property("ro.serialno"))?,
        manufacturer: arg_or_prop(&args.manufacturer, || product_property("manufacturer"))?,
        model: arg_or_prop(&args.model, || product_property("model"))?,
        imei: arg_or_default(&args.imei),
        imei2: arg_or_default(&args.imei2),
        meid: arg_or_default(&args.meid),
    })
}

fn print_attestation_ids(ids: &AttestationIdInfo) {
    for (name, value) in [
        ("brand", &ids.brand),
        ("device", &ids.device),
        ("product", &ids.product),
        ("serial", &ids.serial),
        ("manufacturer", &ids.manufacturer),
        ("model", &ids.model),
        ("imei", &ids.imei),
        ("imei2", &ids.imei2),
        ("meid", &ids.meid),
    ] {
        println!("{name}: {}", String::from_utf8_lossy(value));
    }
}

fn set_attestation_ids(args: &AttestationIdsArgs) -> ExitCode {
    let ids = match collect_attestation_ids(args) {
        Ok(ids) => ids,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };
    print_attestation_ids(&ids);
    if args.dry_run {
        return ExitCode::SUCCESS;
    }

    let req = match (SetAttestationIdsRequest { ids }).into_vec() {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Error: cannot encode the attestation IDs: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    let mut channel = match OpteeChannel::new(KEYMINT_TA_UUID) {
        Ok(channel) => channel,
        Err(e) => {
            eprintln!("Error: cannot open the KeyMint TA: {e}");
            return ExitCode::FAILURE;
        }
    };
    match channel.invoke_with_input(SET_ATTESTATION_IDS, &req) {
        Ok(()) => {
            println!("Attestation IDs provisioned");
            ExitCode::SUCCESS
        }
        Err(TEEC_ERROR_ACCESS_CONFLICT) => {
            eprintln!("Error: the attestation IDs were already provisioned");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("Error: provisioning failed: 0x{e:08x}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    match Cli::parse().action {
        Action::SetAttestationIds(args) => set_attestation_ids(&args),
    }
}
