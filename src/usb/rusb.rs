use crate::error::ChallengeResponseError;
use crate::usb::{Backend, Device, HID_GET_REPORT, HID_SET_REPORT, PRODUCT_ID, REPORT_TYPE_FEATURE, VENDOR_ID};

use rusb::{request_type, Context, DeviceHandle, Direction, Recipient, RequestType, UsbContext};
use std::time::Duration;

pub struct RUSBBackend {
    context: Context,
}

impl Backend<DeviceHandle<Context>, u8> for RUSBBackend {
    fn new() -> Result<Self, ChallengeResponseError> {
        let context = match Context::new() {
            Ok(c) => c,
            Err(e) => return Err(ChallengeResponseError::UsbError(e)),
        };
        Ok(Self { context })
    }

    fn open_device(
        &mut self,
        bus_id: u8,
        address_id: u8,
    ) -> Result<(DeviceHandle<Context>, Vec<u8>), ChallengeResponseError> {
        let devices = match self.context.devices() {
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
    fn close_device(
        &self,
        _handle: DeviceHandle<Context>,
        _interfaces: Vec<u8>,
    ) -> Result<(), ChallengeResponseError> {
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn close_device(
        &self,
        handle: DeviceHandle<Context>,
        interfaces: Vec<u8>,
    ) -> Result<(), ChallengeResponseError> {
        for interface in interfaces {
            handle.release_interface(interface)?;
            handle.attach_kernel_driver(interface)?;
        }
        Ok(())
    }

    fn read(
        &self,
        handle: &mut DeviceHandle<Context>,
        buf: &mut [u8],
    ) -> Result<usize, ChallengeResponseError> {
        assert_eq!(buf.len(), 8);
        let reqtype = request_type(Direction::In, RequestType::Class, Recipient::Interface);
        let value = REPORT_TYPE_FEATURE << 8;
        Ok(handle.read_control(reqtype, HID_GET_REPORT, value, 0, buf, Duration::new(2, 0))?)
    }

    fn raw_write(
        &self,
        handle: &mut DeviceHandle<Context>,
        packet: &[u8],
    ) -> Result<(), ChallengeResponseError> {
        let reqtype = request_type(Direction::Out, RequestType::Class, Recipient::Interface);
        let value = REPORT_TYPE_FEATURE << 8;
        if handle.write_control(reqtype, HID_SET_REPORT, value, 0, &packet, Duration::new(2, 0))? != 8 {
            Err(ChallengeResponseError::CanNotWriteToDevice)
        } else {
            Ok(())
        }
    }

    fn find_device(&mut self) -> Result<Device, ChallengeResponseError> {
        let devices = match self.context.devices() {
            Ok(d) => d,
            Err(e) => return Err(ChallengeResponseError::UsbError(e)),
        };
        for device in devices.iter() {
            let descr = device
                .device_descriptor()
                .map_err(|e| ChallengeResponseError::UsbError(e))?;
            if !VENDOR_ID.contains(&descr.vendor_id()) || !PRODUCT_ID.contains(&descr.product_id()) {
                continue;
            }

            let name = device.open()?.read_product_string_ascii(&descr).ok();
            let serial = self
                .read_serial_from_device(device.bus_number(), device.address())
                .ok();
            let device = Device {
                name,
                serial,
                product_id: descr.product_id(),
                vendor_id: descr.vendor_id(),
                bus_id: device.bus_number(),
                address_id: device.address(),
            };

            return Ok(device);
        }

        Err(ChallengeResponseError::DeviceNotFound)
    }

    fn find_device_from_serial(&mut self, serial: u32) -> Result<Device, ChallengeResponseError> {
        let devices = match self.context.devices() {
            Ok(d) => d,
            Err(e) => return Err(ChallengeResponseError::UsbError(e)),
        };
        for device in devices.iter() {
            let descr = device
                .device_descriptor()
                .map_err(|e| ChallengeResponseError::UsbError(e))?;
            if !VENDOR_ID.contains(&descr.vendor_id()) || !PRODUCT_ID.contains(&descr.product_id()) {
                continue;
            }

            let name = device.open()?.read_product_string_ascii(&descr).ok();
            let fetched_serial = match self
                .read_serial_from_device(device.bus_number(), device.address())
                .ok()
            {
                Some(s) => s,
                None => 0,
            };
            if serial == fetched_serial {
                let device = Device {
                    name,
                    serial: Some(serial),
                    product_id: descr.product_id(),
                    vendor_id: descr.vendor_id(),
                    bus_id: device.bus_number(),
                    address_id: device.address(),
                };

                return Ok(device);
            }
        }

        Err(ChallengeResponseError::DeviceNotFound)
    }

    fn find_all_devices(&mut self) -> Result<Vec<Device>, ChallengeResponseError> {
        let mut result: Vec<Device> = Vec::new();
        let devices = match self.context.devices() {
            Ok(d) => d,
            Err(e) => return Err(ChallengeResponseError::UsbError(e)),
        };
        for device in devices.iter() {
            let descr = device
                .device_descriptor()
                .map_err(|e| ChallengeResponseError::UsbError(e))?;
            if !VENDOR_ID.contains(&descr.vendor_id()) || !PRODUCT_ID.contains(&descr.product_id()) {
                continue;
            }

            let name = device.open()?.read_product_string_ascii(&descr).ok();
            let serial = self
                .read_serial_from_device(device.bus_number(), device.address())
                .ok();
            let device = Device {
                name,
                serial,
                product_id: descr.product_id(),
                vendor_id: descr.vendor_id(),
                bus_id: device.bus_number(),
                address_id: device.address(),
            };
            result.push(device);
        }

        if !result.is_empty() {
            return Ok(result);
        }

        Err(ChallengeResponseError::DeviceNotFound)
    }
}
