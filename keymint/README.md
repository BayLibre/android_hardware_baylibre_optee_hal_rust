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
| `keymint/src/boot_info.rs` | Boot info sent to the TA at startup |
| `keymint/Android.bp` | `android.hardware.security.keymint-service.optee.rust` |
| `keymint/provisioning/src/main.rs` | Tool that provisions the attestation IDs in the TA, once per device |
| `keymint/provisioning/Android.bp` | `keymint-optee-provision-ids` |
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

## Startup Sequence

Before registering the binder services, the HAL sends:

1. `SetBootInfo` (`boot_info.rs`): the Root of Trust and the boot patch level,
   from the properties set by the bootloader (`ro.boot.vbmeta.*`,
   `ro.boot.verifiedbootstate`, `ro.vendor.boot_security_patch`). See
   Limitations.
2. `SetHalInfo` and `SetHalVersion` (`kmr_hal::send_hal_info()`): the OS version
   and patch level and the vendor patch level, from the system properties, and
   the KeyMint AIDL version (4).

The HAL does not send the attestation IDs: the TA reads them from its secure
storage.

## Attestation ID Provisioning

The attestation IDs are provisioned once per device with
`keymint-optee-provision-ids set-attestation-ids`, whose command line and
layout follow the Trusty provisioning tool
(`system/core/trusty/keymint/provisioning`). Each ID can be given as an option
(`--brand`, `--device`, `--product`, `--serial`, `--manufacturer`, `--model`,
`--imei`, `--imei2`, `--meid`). Otherwise, it is read from the same properties
as VTS (`ro.product.<name>_for_attestation`, then `ro.product.vendor.<name>`,
then `ro.product.<name>`, and `ro.serialno`), and a missing property is an
error. The IMEI and MEID are empty unless given (no cellular modem).

The TA stores the IDs and refuses any later provisioning. On a debuggable
build:

```bash
adb root
adb shell keymint-optee-provision-ids set-attestation-ids --dry-run   # print the IDs only
adb shell keymint-optee-provision-ids set-attestation-ids
```

Until then, ID attestation fails with `CANNOT_ATTEST_IDS`, and the RKP
`DeviceInfo` cannot be built.

## Device Integration

Adding the HAL to the product:

- Add `android.hardware.security.keymint-service.optee.rust` to `PRODUCT_PACKAGES`
- Add `keymint-optee-provision-ids` to `PRODUCT_PACKAGES`
- Add SELinux domain of the HAL, with access to `/dev/tee0`

## Compilation

The HAL is built by Soong as part of the product. To build it alone:

```bash
source build/envsetup.sh
lunch <target>
m android.hardware.security.keymint-service.optee.rust keymint-optee-provision-ids
```

Outputs:
- `/vendor/bin/hw/android.hardware.security.keymint-service.optee.rust`
- `vendor/bin/keymint-optee-provision-ids`

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
- `libhex`: vbmeta digests of the boot info
- Android Binder libraries

## Limitations

1. **Boot info**: the Root of Trust and the boot patch level come from
   properties that Android can change, whereas the KeyMint TA expects them
   from the bootloader through a channel that Android cannot subvert
   ("Bootloader" checklist of `system/keymint/README.md`). `boot_info.rs` is
   interim, until the TA gets them from the secure world.
2. **Attestation IDs**: provisioning is not restricted to the factory: any
   process with access to `/dev/tee0` can provision a device that has no IDs
   yet. See the TA `README.md` for the storage limitations.
