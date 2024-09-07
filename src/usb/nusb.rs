use error::ChallengeResponseError;
use nusb::Interface;
use std::time::Duration;
use usb::{DeviceHandle, HID_GET_REPORT, HID_SET_REPORT, REPORT_TYPE_FEATURE};

pub fn open_device(
    _context: &mut (),
    bus_id: u8,
    address_id: u8,
) -> Result<(DeviceHandle, Vec<Interface>), ChallengeResponseError> {
    let nusb_devices = match nusb::list_devices() {
        Ok(d) => d,
        Err(e) => return Err(e.into()),
    };
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

pub fn close_device(
    mut _handle: DeviceHandle,
    _interfaces: Vec<Interface>,
) -> Result<(), ChallengeResponseError> {
    Ok(())
}

pub fn read(handle: &mut DeviceHandle, buf: &mut [u8]) -> Result<usize, ChallengeResponseError> {
    assert_eq!(buf.len(), 8);

    let control_type = nusb::transfer::ControlType::Class;
    let control_in = nusb::transfer::Control {
        control_type,
        recipient: nusb::transfer::Recipient::Interface,
        request: HID_GET_REPORT,
        value: REPORT_TYPE_FEATURE << 8,
        index: 0,
    };

    match handle.control_in_blocking(control_in, buf, Duration::new(2, 0)) {
        Ok(r) => Ok(r),
        Err(_e) => Err(ChallengeResponseError::CanNotReadFromDevice),
    }
}

pub fn raw_write(handle: &mut DeviceHandle, packet: &[u8]) -> Result<(), ChallengeResponseError> {
    let control_type = nusb::transfer::ControlType::Class;
    let control_out = nusb::transfer::Control {
        control_type,
        recipient: nusb::transfer::Recipient::Interface,
        request: HID_SET_REPORT,
        value: REPORT_TYPE_FEATURE << 8,
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
