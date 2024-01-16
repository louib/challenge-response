extern crate challenge_response;
extern crate rand;

use challenge_response::config::{Command, Config};
use challenge_response::configure::DeviceModeConfig;
use challenge_response::hmacmode::HmacKey;
use challenge_response::Yubico;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn main() {
    let mut yubi = Yubico::new();

    if let Ok(device) = yubi.find_yubikey() {
        println!(
            "Vendor ID: {:?} Product ID {:?}",
            device.vendor_id, device.product_id
        );

        let config = Config::default()
            .set_vendor_id(device.vendor_id)
            .set_product_id(device.product_id)
            .set_command(Command::Configuration2);

        let rng = thread_rng();

        let require_press_button = false;

        // Secret must have 20 bytes
        // Used rand here, but you can set your own secret: let secret: &[u8; 20] = b"my_awesome_secret_20";
        let secret: String = rng.sample_iter(&Alphanumeric).take(20).map(char::from).collect();
        let hmac_key: HmacKey = HmacKey::from_slice(secret.as_bytes());

        let mut device_config = DeviceModeConfig::default();
        device_config.challenge_response_hmac(&hmac_key, false, require_press_button);

        if let Err(err) = yubi.write_config(config, &mut device_config) {
            println!("{:?}", err);
        } else {
            println!("Device configured");
        }
    } else {
        println!("Yubikey not found");
    }
}
