extern crate challenge_response;
extern crate hex;

use challenge_response::config::{Config, Mode, Slot};
use challenge_response::ChallengeResponse;

fn main() {
    let mut challenge_response = ChallengeResponse::new().unwrap();

    if let Ok(device) = challenge_response.find_device() {
        println!(
            "Vendor ID: {:?} Product ID {:?}",
            device.vendor_id, device.product_id
        );

        let config = Config::new_from(device).set_mode(Mode::Otp).set_slot(Slot::Slot2);

        // Challenge can not be greater than 64 bytes
        let challenge: &[u8] = b"my_challenge";
        // In OTP Mode, the result will always be different, even if the challenge is the same
        let otp_result = challenge_response
            .challenge_response_otp(challenge, config)
            .unwrap();

        // Just for debug, lets check the hex
        let v: &[u8] = &otp_result.block;
        let hex_string = hex::encode(v);

        println!("{}", hex_string);
    } else {
        println!("Device not found");
    }
}
