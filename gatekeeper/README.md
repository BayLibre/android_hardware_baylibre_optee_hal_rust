# Gatekeeper Rust HAL for OP-TEE

## Overview

This HAL implements the Android Gatekeeper AIDL v1 service with Google's Rust
Gatekeeper HAL library, `gk-hal` (`system/gatekeeper/rust/hal`, `libgk_hal`),
for the Gatekeeper Rust TA running in OP-TEE
(`4d573443-6a56-4272-ac6f-2425af9ef9bb.ta`, sources in
`vendor/baylibre/optee_ta_rust/security_ta/gatekeeper`).

It is based on the AOSP nonsecure Rust Gatekeeper HAL service
(`hardware/interfaces/gatekeeper/aidl/software/rust`). `Checklist.md` tracks
the items of the "HAL Service" checklist of `system/gatekeeper/rust/README.md`.

## Source Layout

| Path | Description |
|------|-------------|
| `gatekeeper/src/gatekeeper_hal_main.rs` | HAL service |
| `gatekeeper/Android.bp` | `android.hardware.gatekeeper-service.optee.rust` |
| `gatekeeper/*.rc`, `gatekeeper/*.xml` | init service and VINTF fragments |
| `gatekeeper/Checklist.md` | "HAL Service" checklist of `system/gatekeeper/rust/README.md` |

## Services Provided

The HAL registers:

1. **IGatekeeper** (`default`): password enrollment, verification, user
   deletion and generation of HardwareAuthTokens consumed by KeyMint
2. **ISharedSecret** (`gatekeeper`): auth token key negotiation, for the
   in-process nonsecure TA

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

## Configuration

The HAL uses:
- **Gatekeeper AIDL version 1** (`IGatekeeper/default`)
- **Init class**: `hal`, running as `system`
