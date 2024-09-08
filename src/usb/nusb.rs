use nusb::{Device as NUSBDevice, Interface};

use error::ChallengeResponseError;
use std::time::Duration;
use usb::{Backend, Device, HID_GET_REPORT, HID_SET_REPORT, PRODUCT_ID, REPORT_TYPE_FEATURE, VENDOR_ID};

pub struct NUSBBackend {}

impl Backend<NUSBDevice, Interface> for NUSBBackend {
    fn new() -> Result<Self, ChallengeResponseError> {
        Ok(Self {})
    }

    fn open_device(
        &mut self,
        bus_id: u8,
        address_id: u8,
    ) -> Result<(NUSBDevice, Vec<Interface>), ChallengeResponseError> {
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

    fn close_device(
        &self,
        mut _handle: NUSBDevice,
        _interfaces: Vec<Interface>,
    ) -> Result<(), ChallengeResponseError> {
        Ok(())
    }

    fn read(&self, handle: &mut NUSBDevice, buf: &mut [u8]) -> Result<usize, ChallengeResponseError> {
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

    fn raw_write(&self, handle: &mut NUSBDevice, packet: &[u8]) -> Result<(), ChallengeResponseError> {
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

    fn find_device(&mut self) -> Result<Device, ChallengeResponseError> {
        match self.find_all_devices() {
            Ok(devices) => {
                if !devices.is_empty() {
                    Ok(devices[0].clone())
                } else {
                    Err(ChallengeResponseError::DeviceNotFound)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn find_device_from_serial(&mut self, serial: u32) -> Result<Device, ChallengeResponseError> {
        let nusb_devices = nusb::list_devices()?;
        for device_info in nusb_devices {
            let product_id = device_info.product_id();
            let vendor_id = device_info.vendor_id();

            if !VENDOR_ID.contains(&vendor_id) || !PRODUCT_ID.contains(&product_id) {
                continue;
            }

            let device_serial =
                match self.read_serial_from_device(device_info.bus_number(), device_info.device_address()) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

            if device_serial == serial {
                return Ok(Device {
                    name: match device_info.manufacturer_string() {
                        Some(name) => Some(name.to_string()),
                        None => Some("unknown".to_string()),
                    },
                    serial: Some(serial),
                    product_id,
                    vendor_id,
                    bus_id: device_info.bus_number(),
                    address_id: device_info.device_address(),
                });
            }
        }
        Err(ChallengeResponseError::DeviceNotFound)
    }

    fn find_all_devices(&mut self) -> Result<Vec<Device>, ChallengeResponseError> {
        let mut devices: Vec<Device> = Vec::new();
        let nusb_devices = nusb::list_devices()?;
        for device_info in nusb_devices {
            let product_id = device_info.product_id();
            let vendor_id = device_info.vendor_id();

            if !VENDOR_ID.contains(&vendor_id) || !PRODUCT_ID.contains(&product_id) {
                continue;
            }

            let device_serial = self
                .read_serial_from_device(device_info.bus_number(), device_info.device_address())
                .ok();

            devices.push(Device {
                name: match device_info.manufacturer_string() {
                    Some(name) => Some(name.to_string()),
                    None => Some("unknown".to_string()),
                },
                serial: device_serial,
                product_id,
                vendor_id,
                bus_id: device_info.bus_number(),
                address_id: device_info.device_address(),
            });
        }
        Ok(devices)
    }
}
