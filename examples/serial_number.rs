extern crate challenge_response;
extern crate hex;

use challenge_response::config::{Config, Slot};
use challenge_response::ChallengeResponse;

fn main() {
    let mut challenge_response = ChallengeResponse::new().unwrap();

    if let Ok(device) = challenge_response.find_device() {
        println!(
            "Vendor ID: {:?} Product ID {:?}",
            device.vendor_id, device.product_id
        );

        let config = Config::new_from(device).set_slot(Slot::Slot2);

        match challenge_response.read_serial_number(config.clone()) {
            Ok(serial_number) => {
                println!("Serial Number {}", serial_number);
            }
            Err(error) => {
                println!("{}", error);
            }
        };

        challenge_response.read_config(config).unwrap();
    } else {
        println!("Device not found");
    }
}
