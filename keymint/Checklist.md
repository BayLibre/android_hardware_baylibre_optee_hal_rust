# KeyMint HAL Service Checklist

Copy of the "HAL Service" section of `system/keymint/README.md`
(android-17.0.0_r1, cfeefc94bf8c4f19d19dca193d376a1fc05e8e3e). Each item is
ticked by the commit that implements it.

### HAL Service

KeyMint appears as a HAL service in userspace, and so an executable that registers for and services
the KeyMint related HALs must be provided.

The implementation of this service is mostly provided by the `kmr-hal` crate, but a driver program
must be provided that:

- Performs start-of-day administration (e.g. logging setup, panic handler setup)
- Creates a communication channel to the KeyMint TA.
- Registers for the KeyMint HAL services.
- Starts a thread pool to service requests.

The KeyMint HAL service (which runs in userspace) must communicate with the KeyMint TA (which runs
in the secure environment).  The reference implementation assumes the existence of a reliable,
message-oriented, bi-directional communication channel for this, as encapsulated in the
`kmr_hal::SerializedChannel` trait.

This trait has a single method `execute()`, which takes as input a request message (as bytes), and
returns a response message (as bytes) or an error.

A (shared) instance of this trait must be provided to each of the `kmr_hal::<interface>::Device`
types, which allows them to service Binder requests for the relevant interface by forwarding the
requests to the TA as request/response pairs.

**Checklist:**

- [x] Implementation of HAL service, which registers for all HAL services.
- [x] SELinux policy for the HAL service.
  (`device/epam/aosp-xenvm-trout/sepolicy/vendor/hal_keymint_optee.te`)
- [x] init.rc configuration for the HAL service.
- [x] Implementation of `SerializedChannel` trait, for reliable HAL <-> TA communication.
  (`libtee_optee_channel` in `vendor/baylibre/optee_ta_rust/optee_channel`)
- [x] Populate userspace environment information at start of day, using `kmr_hal::send_hal_info()`.

The Cuttlefish implementation of the [KeyMint/Rust HAL
service](https://cs.android.com/android/platform/superproject/main/+/main:device/google/cuttlefish/guest/hals/keymint/rust/src/keymint_hal_main.rs)
provides an example of all of the above.
