extern crate challenge_response;
extern crate hex;

use challenge_response::ChallengeResponse;

fn main() {
    let mut challenge_response = ChallengeResponse::new().unwrap();

    let devices = match challenge_response.find_all_devices_nusb() {
        Ok(devices) => devices,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    for device in devices {
        println!(
            "Vendor ID: {:?} Product ID {:?}",
            device.vendor_id, device.product_id
        );
    }
}
