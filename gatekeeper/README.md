# Gatekeeper Rust HAL for OP-TEE

## Overview

This HAL implements the Android Gatekeeper AIDL v1 service with Google's Rust
Gatekeeper HAL library, `gk-hal` (`system/gatekeeper/rust/hal`, `libgk_hal`),
for the Gatekeeper Rust TA running in OP-TEE
(`4d573443-6a56-4272-ac6f-2425af9ef9bb.ta`, sources in
`vendor/baylibre/optee_ta_rust/security_ta/gatekeeper`).

The HAL talks to the TA through `libtee_optee_channel`
(`vendor/baylibre/optee_ta_rust/optee_channel`), over OP-TEE's GlobalPlatform TEE Client API
(`libteec`).

It is based on the AOSP nonsecure Rust Gatekeeper HAL service
(`hardware/interfaces/gatekeeper/aidl/software/rust`). `Checklist.md` tracks
the items of the "HAL Service" checklist of `system/gatekeeper/rust/README.md`.

## Architecture

```
Android Framework (LockSettingsService / gatekeeperd)
    ↓
IGatekeeper (Binder AIDL)
    ↓
android.hardware.gatekeeper-service.optee.rust (this HAL)
    - Uses libgk_hal
    - Talks to the TA through libtee_optee_channel / libteec
    ↓
/dev/tee0 → SMC
    ↓
OP-TEE OS
    ↓
Gatekeeper Rust TA (4d573443-6a56-4272-ac6f-2425af9ef9bb.ta)
    ↓
KeyMint Rust TA (dba51a17-0563-11e7-93b1-6fa7b0071a51.ta)
    - GET_AUTH_TOKEN_KEY (0x10000)
```

## Source Layout

| Path | Description |
|------|-------------|
| `gatekeeper/src/gatekeeper_hal_main.rs` | HAL service |
| `gatekeeper/Android.bp` | `android.hardware.gatekeeper-service.optee.rust` |
| `gatekeeper/*.rc`, `gatekeeper/*.xml` | init service and VINTF fragment |
| `gatekeeper/Checklist.md` | "HAL Service" checklist of `system/gatekeeper/rust/README.md` |

## Services Provided

The HAL registers one service, as `default`:

1. **IGatekeeper**: password enrollment, verification, user deletion and
   generation of HardwareAuthTokens consumed by KeyMint

It does not register the Gatekeeper `ISharedSecret` HAL: the TA gets the auth
token key from the KeyMint TA (see Auth Token Key).

## Communication with the TA

The HAL opens one session to the TA at startup and sends each `gk-wire` (CBOR)
request with a single `PROCESS` command (`OpteeChannel::execute_single()`). The
protocol is described in `vendor/baylibre/optee_ta_rust/optee_channel/README.md`.

Requests are limited to 8000 bytes (`SerializedChannel::MAX_SIZE`), the TA
limit (`GK_MAX_BUFFER_LENGTH` in
`vendor/baylibre/optee_ta_rust/security_ta/gatekeeper/src/main.rs`). Both must stay in sync.

Responses are limited to 4096 bytes (`GK_RESPONSE_MAX` in
`OpteeChannel::execute_single()`), enough for every `gk-wire` response.

The C Gatekeeper TA of other OP-TEE ports uses the same UUID, but not
`gk-wire`: if it is installed instead of the Rust TA, every Gatekeeper
operation fails.

## Device Integration

Adding the HAL to the product:

- Add `android.hardware.gatekeeper-service.optee.rust` to `PRODUCT_PACKAGES`
- Add SELinux domain of the HAL, with access to `/dev/tee0`.

## Compilation

The HAL is built by Soong as part of the product. To build it alone:

```bash
source build/envsetup.sh
lunch <target>
m android.hardware.gatekeeper-service.optee.rust
```

Output: `/vendor/bin/hw/android.hardware.gatekeeper-service.optee.rust`

See the TA `README.md` for the TA build details. The KeyMint TA must also be
installed, as Gatekeeper gets the auth token key from it (see
`vendor/baylibre/optee_ta_rust/security_ta/keymint`).

## Configuration

The HAL uses:
- **TA UUID**: `4d573443-6a56-4272-ac6f-2425af9ef9bb`
- **Gatekeeper AIDL version 1** (`IGatekeeper/default`)
- **Init class**: `hal`, running as `system`

### Auth Token Key

The HMAC-SHA256 key that signs the HardwareAuthTokens is the one KeyMint agrees
through its `ISharedSecret` HAL. The Gatekeeper TA gets it from the KeyMint TA
(`GET_AUTH_TOKEN_KEY`) at each signature, and fails to sign until the
negotiation has run.

## Debugging

HAL logs: `adb logcat -s gatekeeper-hal-optee-rust`, at debug level on
debuggable builds (`ro.debuggable`), info level otherwise.

VTS: `VtsHalGatekeeperTargetTest`.

## Dependencies

- `libgk_hal`: Google Gatekeeper HAL library, with the Gatekeeper AIDL
  interface
- `libtee_optee_channel` (`vendor/baylibre/optee_ta_rust/optee_channel`) / `libteec`
  (`vendor/optee/optee_client`): OP-TEE Client API
- `librustutils`: system properties
- Android Binder libraries
