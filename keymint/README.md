# KeyMint Rust HAL for OP-TEE

## Overview

This HAL implements the Android KeyMint v4 services with Google's Rust KeyMint
HAL library, `kmr-hal` (`system/keymint/hal`, `libkmr_hal_v4`), for the KeyMint
Rust TA running in OP-TEE (`dba51a17-0563-11e7-93b1-6fa7b0071a51.ta`, sources
in `vendor/baylibre/optee_ta_rust/security_ta/keymint`).

It is based on the AOSP Trusty KeyMint HAL service
(`system/core/trusty/keymint`). `Checklist.md` tracks the items of the "HAL
Service" checklist of `system/keymint/README.md`.

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

## Configuration

The HAL uses:
- **KeyMint AIDL version 4** (`libkmr_hal_v4`), with
  `android.hardware.hardware_keystore_V4.xml`
- **Init class**: `early_hal`, running as `system`. Like the Trusty service, it
  is `oneshot`: if it crashes, the device must be restarted.
