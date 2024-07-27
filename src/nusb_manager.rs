use error::ChallengeResponseError;
use manager::{Flags, Frame};
use nusb::{Device, Interface};
use std::time::Duration;
use std::{slice, thread};

pub fn open_device(bus_id: u8, address_id: u8) -> Result<(Device, Vec<Interface>), ChallengeResponseError> {
    let nusb_devices = nusb::list_devices();
    // FIXME remove this unwrap
    let nusb_devices = nusb_devices.unwrap();
    for device_info in nusb_devices {
        if device_info.bus_number() != bus_id || device_info.device_address() != address_id {
            continue;
        }

        let device = match device_info.open() {
            Ok(d) => d,
            Err(_) => {
                return Err(ChallengeResponseError::OpenDeviceError);
            }
        };
        // let config = &device.configurations().collect::<Vec<_>>()[0];
        // match device.active_configuration() {
        //     Ok(active_config) => {
        //         if active_config.configuration_value() != config.configuration_value() {
        //             println!("Setting config value");
        //             device.set_configuration(config.configuration_value())?;
        //         }
        //         device.set_configuration(config.configuration_value())?;
        //         println!("Device configuration is already active");
        //     }
        //     Err(_) => {
        //         println!("Setting config value");
        //         device.set_configuration(config.configuration_value())?;
        //     }
        // };

        let mut interfaces: Vec<Interface> = Vec::new();
        for interface in device_info.interfaces() {
            let interface = match device.detach_and_claim_interface(interface.interface_number()) {
                Ok(interface) => interface,
                Err(_) => continue,
            };

            interfaces.push(interface);
        }
        return Ok((device, interfaces));
    }

    Err(ChallengeResponseError::DeviceNotFound)
}

pub fn close_device(mut _handle: Device, _interfaces: Vec<Interface>) -> Result<(), ChallengeResponseError> {
    Ok(())
}

pub fn wait<F: Fn(Flags) -> bool>(
    handle: &mut Device,
    f: F,
    buf: &mut [u8],
) -> Result<(), ChallengeResponseError> {
    loop {
        read(handle, buf)?;
        let flags = Flags::from_bits_truncate(buf[7]);
        if flags.contains(Flags::SLOT_WRITE_FLAG) || flags.is_empty() {
            // Should store the version
        }

        if f(flags) {
            return Ok(());
        }
        thread::sleep(Duration::new(0, 1000000));
    }
}

pub fn read(handle: &mut Device, buf: &mut [u8]) -> Result<usize, ChallengeResponseError> {
    assert_eq!(buf.len(), 8);

    let control_type = nusb::transfer::ControlType::Class;
    let control_in = nusb::transfer::Control {
        control_type,
        recipient: nusb::transfer::Recipient::Interface,
        request: crate::manager::HID_GET_REPORT,
        value: crate::manager::REPORT_TYPE_FEATURE << 8,
        index: 0,
    };

    match handle.control_in_blocking(control_in, buf, Duration::new(2, 0)) {
        Ok(r) => Ok(r),
        Err(_e) => Err(ChallengeResponseError::CanNotReadFromDevice),
    }
}

pub fn write_frame(handle: &mut Device, frame: &Frame) -> Result<(), ChallengeResponseError> {
    let mut data = unsafe { slice::from_raw_parts(frame as *const Frame as *const u8, 70) };

    let mut seq = 0;
    let mut buf = [0; 8];
    while !data.is_empty() {
        let (a, b) = data.split_at(7);

        if seq == 0 || b.is_empty() || a.iter().any(|&x| x != 0) {
            let mut packet = [0; 8];
            (&mut packet[..7]).copy_from_slice(a);

            packet[7] = Flags::SLOT_WRITE_FLAG.bits() + seq;
            wait(handle, |x| !x.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
            raw_write(handle, &packet)?;
        }
        data = b;
        seq += 1
    }
    Ok(())
}

pub fn raw_write(handle: &mut Device, packet: &[u8]) -> Result<(), ChallengeResponseError> {
    let control_type = nusb::transfer::ControlType::Class;
    let control_out = nusb::transfer::Control {
        control_type,
        recipient: nusb::transfer::Recipient::Interface,
        request: crate::manager::HID_SET_REPORT,
        value: crate::manager::REPORT_TYPE_FEATURE << 8,
        index: 0,
    };

    match handle.control_out_blocking(control_out, packet, Duration::new(2, 0)) {
        Ok(bytes_written) => {
            if bytes_written != 8 {
                Err(ChallengeResponseError::CanNotWriteToDevice)
            } else {
                Ok(())
            }
        }
        Err(_) => Err(ChallengeResponseError::CanNotWriteToDevice),
    }
}

/// Reset the write state after a read.
pub fn write_reset(handle: &mut Device) -> Result<(), ChallengeResponseError> {
    raw_write(handle, &[0, 0, 0, 0, 0, 0, 0, 0x8f])?;
    let mut buf = [0; 8];
    wait(handle, |x| !x.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
    Ok(())
}

pub fn read_response(handle: &mut Device, response: &mut [u8]) -> Result<usize, ChallengeResponseError> {
    let mut r0 = 0;
    wait(
        handle,
        |f| f.contains(Flags::RESP_PENDING_FLAG),
        &mut response[..8],
    )?;
    r0 += 7;
    loop {
        if read(handle, &mut response[r0..r0 + 8])? < 8 {
            break;
        }
        let flags = Flags::from_bits_truncate(response[r0 + 7]);
        if flags.contains(Flags::RESP_PENDING_FLAG) {
            let seq = response[r0 + 7] & 0b00011111;
            if r0 > 0 && seq == 0 {
                // If the sequence number is 0, and we have read at
                // least one packet, stop.
                break;
            }
        } else {
            break;
        }
        r0 += 7;
    }
    write_reset(handle)?;
    Ok(r0)
}
