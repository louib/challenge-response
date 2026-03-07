# challenge-response

[![Latest Version]][crates.io] [![Documentation]][docs.rs] [![Build Status]][CI on Master] [![Dependency Status]][deps.rs] [![MIT licensed]][MIT] [![Apache-2.0 licensed]][APACHE]

[Documentation]: https://docs.rs/challenge_response/badge.svg
[docs.rs]: https://docs.rs/challenge-response/
[Latest Version]: https://img.shields.io/crates/v/challenge-response.svg
[crates.io]: https://crates.io/crates/challenge-response
[MIT licensed]: https://img.shields.io/badge/License-MIT-blue.svg
[MIT]: ./LICENSE-MIT
[Apache-2.0 licensed]: https://img.shields.io/badge/License-Apache%202.0-blue.svg
[APACHE]: ./LICENSE-APACHE
[Dependency Status]: https://deps.rs/repo/github/louib/challenge-response/status.svg
[deps.rs]: https://deps.rs/repo/github/louib/challenge-response
[Build Status]: https://github.com/louib/challenge-response/actions/workflows/merge.yml/badge.svg?branch=master
[CI on Master]: https://github.com/louib/challenge-response/actions/workflows/merge.yml

`challenge-response` is a Rust library for performing [challenge-response](https://wiki.archlinux.org/index.php/yubikey#Function_and_Application_of_Challenge-Response) operations (hashing and encryption) using security keys like the YubiKey and the OnlyKey.

## Examples

Code examples for performing a challenge-response (both HMAC and OTP) as well as configuring a device can be found in the [examples/](https://github.com/louib/challenge-response/tree/master/examples) directory.

## Current features

- [HMAC-SHA1 Challenge-Response](https://datatracker.ietf.org/doc/html/rfc2104)
- [Yubico OTP Challenge-Response encryption](https://docs.yubico.com/yesdk/users-manual/application-otp/yubico-otp.html)
- Challenge-Response configuration

## Supported devices

- YubiKey 2.2 and later
- OnlyKey (**untested**)
- NitroKey (**untested**)

## Usage

Add this to your `Cargo.toml`

```toml
[dependencies]
challenge_response = "0"
```

### nusb backend (EXPERIMENTAL)

You can enable the experimental [nusb](https://crates.io/crates/nusb) backend by adding the following to your `Cargo.toml` manifest:

```toml
[dependencies]
challenge_response = { version = "0", default-features = false, features = ["nusb"] }
```

The `nusb` backend has the advantage of not depending on `libusb`, thus making it easier to add
`challenge_response` to your dependencies.

> [!NOTE]
> The `nusb` feature is not available on Windows. If configured, the library will default to using the `rusb` backend instead.

## Credits

This library was originally a fork of the [yubico_manager](https://crates.io/crates/yubico_manager) library.

## License

MIT or Apache-2.0
