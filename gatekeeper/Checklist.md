# Gatekeeper HAL Service Checklist

Copy of the "HAL Service" section of `system/gatekeeper/rust/README.md`
(android-17.0.0_r1, 5b5e75b5bda9fccbc3132e9624cb25286babdaac). Each item is
ticked by the commit that implements it.

### HAL Service

Gatekeeper appears as a HAL service in userspace, and so an executable that registers for and
services the Gatekeeper related HALs must be provided.

The implementation of this service is mostly provided by the `gk-hal` crate, but a driver program
must be provided that:

- Performs start-of-day administration (e.g. logging setup, panic handler setup)
- Creates a communication channel to the Gatekeeper TA.
- Registers for the Gatekeeper HAL services.
- Starts a thread pool to service requests.

The Gatekeeper HAL service (which runs in userspace) must communicate with the Gatekeeper TA (which
runs in the secure environment).  The reference implementation assumes the existence of a reliable,
message-oriented, bi-directional communication channel for this, as encapsulated in the
`gk_hal::SerializedChannel` trait.

This trait has a single method `execute()`, which takes as input a request message (as bytes), and
returns a response message (as bytes) or an error.

**Checklist:**

- [ ] Implementation of HAL service main executable.
- [ ] SELinux policy for the HAL service.
- [ ] init.rc configuration for the HAL service.
- [ ] Implementation of `SerializedChannel` trait, for reliable HAL <-> TA communication.

The <a
href="https://cs.android.com/android/platform/superproject/main/+/main:system/core/trusty/gatekeeper/rust/">Trusty-specific
implementation</a> provides an example of this.
