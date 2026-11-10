# KeyMint Rust HAL for OP-TEE

## Overview

This HAL implements the Android KeyMint v4 services with Google's Rust KeyMint
HAL library, `kmr-hal` (`system/keymint/hal`, `libkmr_hal_v4`), for the KeyMint
Rust TA running in OP-TEE (`dba51a17-0563-11e7-93b1-6fa7b0071a51.ta`, sources
in `vendor/baylibre/optee_ta_rust/security_ta/keymint`).

The HAL talks to the TA through `libtee_optee_channel`
(`vendor/baylibre/optee_ta_rust/optee_channel`), over OP-TEE's GlobalPlatform TEE Client API
(`libteec`).

It is based on the AOSP Trusty KeyMint HAL service
(`system/core/trusty/keymint`). `Checklist.md` tracks the items of the "HAL
Service" checklist of `system/keymint/README.md`.

## Architecture

```
Android Framework (keystore2)
    ↓
IKeyMintDevice, IRemotelyProvisionedComponent,
ISecureClock, ISharedSecret (Binder AIDL)
    ↓
android.hardware.security.keymint-service.optee.rust (this HAL)
    - Uses libkmr_hal_v4
    - Talks to the TA through libtee_optee_channel / libteec
    ↓
/dev/tee0 → SMC
    ↓
OP-TEE OS
    ↓
KeyMint Rust TA (dba51a17-0563-11e7-93b1-6fa7b0071a51.ta)
```

## Source Layout

| Path | Description |
|------|-------------|
| `keymint/src/keymint_hal_main.rs` | HAL service |
| `keymint/Android.bp` | `android.hardware.security.keymint-service.optee.rust` |
| `keymint/*.rc`, `keymint/*.xml` | init service and VINTF fragment |
| `keymint/Checklist.md` | "HAL Service" checklist of `system/keymint/README.md` |

## Services Provided

The HAL registers the four KeyMint services (`kmr_hal::ALL_HALS`), as
`default`:

1. **IKeyMintDevice** (v4): key generation, import, encryption, signing,
   attestation
2. **IRemotelyProvisionedComponent** (v3): remote key provisioning
3. **ISecureClock**: secure timestamps
4. **ISharedSecret**: shared secret negotiation

## Communication with the TA

The HAL opens one session to the TA at startup, shared by the four services.
Each `kmr-wire` (CBOR) request is sent with `WRITE` and the response is read
back in chunks with `READ` (`OpteeChannel::execute()`). The protocol is
described in `vendor/baylibre/optee_ta_rust/optee_channel/README.md`.

Requests are limited to 4000 bytes (`SerializedChannel::MAX_SIZE`, as in the
Trusty HAL), below the TA limit of 4096 bytes (`KEYMINT_MAX_BUFFER_LENGTH`).

## Device Integration

Adding the HAL to the product:

- Add `android.hardware.security.keymint-service.optee.rust` to `PRODUCT_PACKAGES`
- Add SELinux domain of the HAL, with access to `/dev/tee0`

## Compilation

The HAL is built by Soong as part of the product. To build it alone:

```bash
source build/envsetup.sh
lunch <target>
m android.hardware.security.keymint-service.optee.rust
```

Output: `/vendor/bin/hw/android.hardware.security.keymint-service.optee.rust`

See the TA `README.md` for the TA build details.

## Configuration

The HAL uses:
- **TA UUID**: `dba51a17-0563-11e7-93b1-6fa7b0071a51`
- **KeyMint AIDL version 4** (`libkmr_hal_v4`), with
  `android.hardware.hardware_keystore_V4.xml`
- **Init class**: `early_hal`, running as `system`. Like the Trusty service, it
  is `oneshot`: if it crashes, the device must be restarted.

## Debugging

HAL logs: `adb logcat -s keymint-hal-optee-rust`, at debug level on
debuggable builds (`ro.debuggable`), info level otherwise.

VTS: `VtsAidlKeyMintTargetTest`, `VtsAidlSecureClockTargetTest`,
`VtsAidlSharedSecretTargetTest`

## Dependencies

- `libkmr_hal_v4`, `libkmr_wire`: Google KeyMint HAL library and wire format,
  with the KeyMint, RKP, SecureClock and SharedSecret AIDL interfaces
- `libtee_optee_channel` (`vendor/baylibre/optee_ta_rust/optee_channel`) / `libteec`
  (`vendor/optee/optee_client`): OP-TEE Client API
- `librustutils`: system properties
- Android Binder libraries
