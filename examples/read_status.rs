use challenge_response::{config::Config, ChallengeResponse};

fn main() {
    let mut cr = ChallengeResponse::new().expect("Failed to initialize ChallengeResponse");

    match cr.find_device() {
        Ok(device) => {
            println!("Found device: {:?}", device.name.as_deref().unwrap_or("Unknown"));
            println!("Serial: {:?}", device.serial);

            let conf = Config::new_from(device);
            match cr.read_status(conf) {
                Ok(status) => {
                    println!("--- Device Status ---");
                    println!(
                        "Firmware Version: {}.{}.{}",
                        status.version_major, status.version_minor, status.version_build
                    );
                    println!("Programming Sequence: {}", status.pgm_seq);
                    println!("Touch Level: 0x{:04x}", status.touch_level);

                    let slot1_configured = (status.touch_level & 1) != 0;
                    let slot2_configured = (status.touch_level & 2) != 0;

                    println!(
                        "Slot 1: {}",
                        if slot1_configured {
                            "Configured"
                        } else {
                            "Not configured"
                        }
                    );
                    println!(
                        "Slot 2: {}",
                        if slot2_configured {
                            "Configured"
                        } else {
                            "Not configured"
                        }
                    );
                }
                Err(e) => eprintln!("Error reading status: {:?}", e),
            }
        }
        Err(e) => eprintln!("No device found: {:?}", e),
    }
}
