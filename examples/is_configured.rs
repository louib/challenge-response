use challenge_response::{config::Slot, ChallengeResponse, Device};

fn main() {
    let mut cr = ChallengeResponse::new().unwrap();
    let device = cr.find_device().unwrap();

    println!("Checking configuration for device: {:?}", device.name);

    check_slot(&mut cr, &device, Slot::Slot1);
    check_slot(&mut cr, &device, Slot::Slot2);
}

fn check_slot(cr: &mut ChallengeResponse, device: &Device, slot: Slot) {
    match cr.is_configured(device.clone(), slot.clone()) {
        Ok(configured) => {
            println!(
                "Slot {:?} is {}configured",
                slot,
                if configured { "" } else { "not " }
            );
        }
        Err(e) => {
            eprintln!("Error checking slot {:?}: {:?}", slot, e);
        }
    }
}
