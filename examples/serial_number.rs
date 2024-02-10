extern crate challenge_response;
extern crate hex;

use challenge_response::config::{Config, Slot};
use challenge_response::Yubico;

fn main() {
    let mut yubi = Yubico::new().unwrap();

    if let Ok(device) = yubi.find_yubikey() {
        println!(
            "Vendor ID: {:?} Product ID {:?}",
            device.vendor_id, device.product_id
        );

        let config = Config::new_from(device).set_slot(Slot::Slot2);

        match yubi.read_serial_number(config) {
            Ok(serial_number) => {
                println!("Serial Number {}", serial_number);
            }
            Err(error) => {
                println!("{}", error);
            }
        };
    } else {
        println!("Yubikey not found");
    }
}
