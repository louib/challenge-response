use error::ChallengeResponseError;
use rusb::{request_type, Context, Direction, Recipient, RequestType, UsbContext};
use std::time::Duration;
use usb::{DeviceHandle, HID_GET_REPORT, HID_SET_REPORT, REPORT_TYPE_FEATURE};

pub fn open_device(
    context: &mut Context,
    bus_id: u8,
    address_id: u8,
) -> Result<(DeviceHandle, Vec<u8>), ChallengeResponseError> {
    let devices = match context.devices() {
        Ok(device) => device,
        Err(_) => {
            return Err(ChallengeResponseError::DeviceNotFound);
        }
    };

    for device in devices.iter() {
        match device.device_descriptor() {
            Ok(_) => {}
            Err(_) => {
                return Err(ChallengeResponseError::DeviceNotFound);
            }
        };

        if device.bus_number() == bus_id && device.address() == address_id {
            match device.open() {
                Ok(handle) => {
                    let config = match device.config_descriptor(0) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let mut _interfaces = Vec::new();
                    for interface in config.interfaces() {
                        for usb_int in interface.descriptors() {
                            match handle.kernel_driver_active(usb_int.interface_number()) {
                                Ok(true) => {
                                    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                                    handle.detach_kernel_driver(usb_int.interface_number())?;
                                }
                                _ => continue,
                            };

                            if handle.active_configuration()? != config.number() {
                                handle.set_active_configuration(config.number())?;
                            }
                            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                            handle.claim_interface(usb_int.interface_number())?;
                            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                            _interfaces.push(usb_int.interface_number());
                        }
                    }

                    return Ok((handle, _interfaces));
                }
                Err(_) => {
                    return Err(ChallengeResponseError::OpenDeviceError);
                }
            }
        }
    }

    Err(ChallengeResponseError::DeviceNotFound)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn close_device(_handle: DeviceHandle, _interfaces: Vec<u8>) -> Result<(), ChallengeResponseError> {
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn close_device(handle: DeviceHandle, interfaces: Vec<u8>) -> Result<(), ChallengeResponseError> {
    for interface in interfaces {
        handle.release_interface(interface)?;
        handle.attach_kernel_driver(interface)?;
    }
    Ok(())
}

pub fn read(handle: &mut DeviceHandle, buf: &mut [u8]) -> Result<usize, ChallengeResponseError> {
    assert_eq!(buf.len(), 8);
    let reqtype = request_type(Direction::In, RequestType::Class, Recipient::Interface);
    let value = REPORT_TYPE_FEATURE << 8;
    Ok(handle.read_control(reqtype, HID_GET_REPORT, value, 0, buf, Duration::new(2, 0))?)
}

pub fn raw_write(handle: &mut DeviceHandle, packet: &[u8]) -> Result<(), ChallengeResponseError> {
    let reqtype = request_type(Direction::Out, RequestType::Class, Recipient::Interface);
    let value = REPORT_TYPE_FEATURE << 8;
    if handle.write_control(reqtype, HID_SET_REPORT, value, 0, &packet, Duration::new(2, 0))? != 8 {
        Err(ChallengeResponseError::CanNotWriteToDevice)
    } else {
        Ok(())
    }
}
