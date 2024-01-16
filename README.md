# challenge-response [![Latest Version]][crates.io] [![MIT licensed]][MIT] [![Apache-2.0 licensed]][APACHE]

[Latest Version]: https://img.shields.io/crates/v/challenge-response.svg
[crates.io]: https://crates.io/crates/challenge-response
[MIT licensed]: https://img.shields.io/badge/License-MIT-blue.svg
[MIT]: ./LICENSE-MIT
[Apache-2.0 licensed]: https://img.shields.io/badge/License-Apache%202.0-blue.svg
[APACHE]: ./LICENSE-APACHE

**Challenge-Response library for Rust**

---

## Current features

- [X] [Challenge-Response](https://wiki.archlinux.org/index.php/yubikey#Function_and_Application_of_Challenge-Response), YubiKey 2.2 and later supports HMAC-SHA1 or Yubico challenge-response operations.
- [x] Configuration.

## Usage

Add this to your Cargo.toml

```toml
[dependencies]
challenge_response = "0"
```

### Configure Yubikey (HMAC-SHA1 mode)

Note, please read about the [initial configuration](https://wiki.archlinux.org/index.php/yubikey#Initial_configuration)
Alternatively you can configure the yubikey with the official [Yubikey Personalization GUI](https://developers.yubico.com/yubikey-personalization-gui/).

```rust
extern crate rand;
extern crate yubico_manager;

use yubico_manager::{Yubico};
use yubico_manager::config::{Config, Command};
use yubico_manager::configure::{ DeviceModeConfig };
use yubico_manager::hmacmode::{ HmacKey };
use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric};

fn main() {
   let mut yubi = Yubico::new();

   if let Ok(device) = yubi.find_yubikey() {
       println!("Vendor ID: {:?} Product ID {:?}", device.vendor_id, device.product_id);

       let config = Config::default()
           .set_vendor_id(device.vendor_id)
           .set_product_id(device.product_id)
           .set_command(Command::Configuration2);

        let mut rng = thread_rng();

        // Secret must have 20 bytes
        // Used rand here, but you can set your own secret: let secret: &[u8; 20] = b"my_awesome_secret_20";
        let secret: String = rng.sample_iter(&Alphanumeric).take(20).collect();
        let hmac_key: HmacKey = HmacKey::from_slice(secret.as_bytes());

        let mut device_config = DeviceModeConfig::default();
        device_config.challenge_response_hmac(&hmac_key, false, false);

        if let Err(err) = yubi.write_config(config, &mut device_config) {
            println!("{:?}", err);
        } else {
            println!("Device configured");
        }

   } else {
       println!("Yubikey not found");
   }
}
```

### Example Challenge-Response (HMAC-SHA1 mode)

Configure the yubikey with [Yubikey Personalization GUI](https://developers.yubico.com/yubikey-personalization-gui/)

```rust
extern crate hex;
extern crate yubico_manager;

use std::ops::Deref;
use yubico_manager::{Yubico};
use yubico_manager::config::{Config, Slot, Mode};

fn main() {
   let mut yubi = Yubico::new();

   if let Ok(device) = yubi.find_yubikey() {
       println!("Vendor ID: {:?} Product ID {:?}", device.vendor_id, device.product_id);

       let config = Config::default()
           .set_vendor_id(device.vendor_id)
           .set_product_id(device.product_id)
           .set_variable_size(true)
           .set_mode(Mode::Sha1)
           .set_slot(Slot::Slot2);

       // Challenge can not be greater than 64 bytes
       let challenge = String::from("mychallenge");
       // In HMAC Mode, the result will always be the SAME for the SAME provided challenge
       let hmac_result= yubi.challenge_response_hmac(challenge.as_bytes(), config).unwrap();

       // Just for debug, lets check the hex
       let v: &[u8] = hmac_result.deref();
       let hex_string = hex::encode(v);

       println!("{}", hex_string);

   } else {
       println!("Yubikey not found");
   }
}
```
