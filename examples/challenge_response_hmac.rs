extern crate challenge_response;
extern crate hex;

use challenge_response::config::{Config, Mode, Slot};
use challenge_response::Yubico;
use std::ops::Deref;

fn main() {
    let mut yubi = Yubico::new();

    if let Ok(devices) = yubi.find_all_yubikeys() {
        let device = devices[0].clone();
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
        // In HMAC Mode, the result will always be the SAME for the SAME provided challenge
        let hmac_result = yubi
            .challenge_response_hmac(challenge.as_bytes(), config)
            .unwrap();

        // Just for debug, lets check the hex
        let v: &[u8] = hmac_result.deref();
        let hex_string = hex::encode(v);

        println!("{}", hex_string);
    } else {
        println!("Yubikey not found");
    }
}
