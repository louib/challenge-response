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

### Perform a Challenge-Response (HMAC-SHA1 mode)

If you are using a YubiKey, you can configure the HMAC-SHA1 Challenge-Response
with the [Yubikey Personalization GUI](https://developers.yubico.com/yubikey-personalization-gui/).

```rust,ignore
extern crate challenge_response;
extern crate hex;

use challenge_response::config::{Config, Mode, Slot};
use challenge_response::ChallengeResponse;
use std::ops::Deref;

fn main() {
    let mut cr_client = match ChallengeResponse::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return;
        }
    };

    let device = match cr_client.find_device() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Device not found: {}", e.to_string());
            return;
        }
    };

    println!(
        "Vendor ID: {:?} Product ID {:?}",
        device.vendor_id, device.product_id
    );

    let config = Config::new_from(device)
        .set_variable_size(true)
        .set_mode(Mode::Sha1)
        .set_slot(Slot::Slot2);

    // Challenge can not be greater than 64 bytes
    let challenge = String::from("mychallenge");
    // In HMAC Mode, the result will always be the
    // SAME for the SAME provided challenge
    let hmac_result = cr_client
        .challenge_response_hmac(challenge.as_bytes(), config)
        .unwrap();

    // Just for debug, lets check the hex
    let v: &[u8] = hmac_result.deref();
    let hex_string = hex::encode(v);

    println!("{}", hex_string);
}
```

### Configure Yubikey (HMAC-SHA1 mode)

Note, please read about the [initial configuration](https://wiki.archlinux.org/index.php/yubikey#Initial_configuration)
Alternatively you can configure the yubikey with the official [Yubikey Personalization GUI](https://developers.yubico.com/yubikey-personalization-gui/).

```rust,ignore
extern crate challenge_response;
extern crate rand;

use challenge_response::config::{Command, Config};
use challenge_response::configure::DeviceModeConfig;
use challenge_response::hmacmode::{
    HmacKey, HmacSecret, HMAC_SECRET_SIZE,
};
use challenge_response::ChallengeResponse;
use rand::distr::Alphanumeric;
use rand::{rng, Rng};

fn main() {
    let mut cr_client = match ChallengeResponse::new() {
        Ok(y) => y,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return;
        }
    };

    let device = match cr_client.find_device() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Device not found: {}", e.to_string());
            return;
        }
    };

    println!(
        "Vendor ID: {:?} Product ID {:?}",
        device.vendor_id, device.product_id
    );

    let config = Config::new_from(device)
        .set_command(Command::Configuration2);

    let mut rng = rng();

    // Used rand here, but you can set your own secret:
    // let secret: &HmacSecret = b"my_awesome_secret_20";
    let secret: Vec<u8> = rng
        .sample_iter(&Alphanumeric)
        .take(HMAC_SECRET_SIZE)
        .collect();
    let hmac_key: HmacKey = HmacKey::from_slice(&secret);

    let mut device_config = DeviceModeConfig::default();
    device_config.challenge_response_hmac(&hmac_key, false, false);

    if let Err(err) =
        cr_client.write_config(config, &mut device_config)
    {
        println!("{:?}", err);
    } else {
        println!("Device configured");
    }
}
```

## Credits

This library was originally a fork of the [yubico_manager](https://crates.io/crates/yubico_manager) library.

## License

MIT or Apache-2.0
