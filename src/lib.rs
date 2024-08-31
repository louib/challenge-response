#![doc = include_str!("../README.md")]
extern crate rusb;

#[macro_use]
extern crate structure;

extern crate aes;
extern crate block_modes;
extern crate hmac;
extern crate rand;
extern crate sha1;
#[macro_use]
extern crate bitflags;

pub mod config;
pub mod configure;
pub mod error;
pub mod hmacmode;
pub mod otpmode;
mod sec;
mod usb;

use aes::cipher::generic_array::GenericArray;

use config::Command;
use config::{Config, Slot};
use configure::DeviceModeConfig;
use error::ChallengeResponseError;
use hmacmode::Hmac;
use otpmode::Aes128Block;
use rusb::UsbContext;
use sec::{crc16, CRC_RESIDUAL_OK};
use usb::{close_device, open_device, read_response, wait, write_frame, Context, Flags, Frame};

const VENDOR_ID: [u16; 3] = [
    0x1050, // Yubico ( Yubikeys )
    0x1D50, // OpenMoko ( Onlykey )
    0x20A0, // Flirc ( Nitrokey )
];
const PRODUCT_ID: [u16; 11] = [
    0x0010, // YubiKey Gen 1 & 2
    0x0110, 0x0113, 0x0114, 0x0116, // YubiKey NEO
    0x0401, 0x0403, 0x0405, 0x0407, // Yubikey 4 & 5
    0x60FC, // Onlykey
    0x4211, // NitroKey
];

/// If using a variable-length challenge, the challenge must be stricly smaller than this value.
/// If using a fixed-length challenge, the challenge must be exactly equal to this value.
pub const CHALLENGE_SIZE: usize = 64;

/// The `Result` type used in this crate.
type Result<T> = ::std::result::Result<T, ChallengeResponseError>;

#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub name: Option<String>,
    pub serial: Option<u32>,
    pub product_id: u16,
    pub vendor_id: u16,
    pub bus_id: u8,
    pub address_id: u8,
}

pub struct ChallengeResponse {
    context: Context,
}

impl ChallengeResponse {
    /// Creates a new ChallengeResponse instance.
    pub fn new() -> Result<Self> {
        let context = match Context::new() {
            Ok(c) => c,
            Err(e) => return Err(ChallengeResponseError::UsbError(e)),
        };
        Ok(ChallengeResponse { context })
    }

    fn read_serial_from_device(&mut self, device: rusb::Device<Context>) -> Result<u32> {
        let (mut handle, interfaces) = open_device(&mut self.context, device.bus_number(), device.address())?;
        let challenge = [0; CHALLENGE_SIZE];
        let command = Command::DeviceSerial;

        let d = Frame::new(challenge, command); // FixMe: do not need a challange
        let mut buf = [0; usb::STATUS_UPDATE_PAYLOAD_SIZE];
        wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;

        write_frame(&mut handle, &d)?;

        // Read the response.
        let mut response = [0; usb::RESPONSE_SIZE];
        read_response(&mut handle, &mut response)?;
        close_device(handle, interfaces)?;

        // Check response.
        if crc16(&response[..6]) != crate::sec::CRC_RESIDUAL_OK {
            return Err(ChallengeResponseError::WrongCRC);
        }

        let serial = structure!("2I").unpack(response[..8].to_vec())?;

        Ok(serial.0)
    }

    pub fn find_device(&mut self) -> Result<Device> {
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
            let serial = self.read_serial_from_device(device.clone()).ok();
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

    pub fn find_device_from_serial(&mut self, serial: u32) -> Result<Device> {
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
            let fetched_serial = match self.read_serial_from_device(device.clone()).ok() {
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

    pub fn find_all_devices(&mut self) -> Result<Vec<Device>> {
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
            let serial = self.read_serial_from_device(device.clone()).ok();
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

    pub fn write_config(&mut self, conf: Config, device_config: &mut DeviceModeConfig) -> Result<()> {
        let d = device_config.to_frame(conf.command);
        let mut buf = [0; usb::STATUS_UPDATE_PAYLOAD_SIZE];

        let (mut handle, interfaces) =
            open_device(&mut self.context, conf.device.bus_id, conf.device.address_id)?;

        wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;

        // TODO: Should check version number.

        write_frame(&mut handle, &d)?;
        wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
        close_device(handle, interfaces)?;

        Ok(())
    }

    pub fn read_serial_number(&mut self, conf: Config) -> Result<u32> {
        let (mut handle, interfaces) =
            open_device(&mut self.context, conf.device.bus_id, conf.device.address_id)?;

        let challenge = [0; CHALLENGE_SIZE];
        let command = Command::DeviceSerial;

        let d = Frame::new(challenge, command); // FixMe: do not need a challange
        let mut buf = [0; usb::STATUS_UPDATE_PAYLOAD_SIZE];
        wait(
            &mut handle,
            |f| !f.contains(usb::Flags::SLOT_WRITE_FLAG),
            &mut buf,
        )?;

        write_frame(&mut handle, &d)?;

        // Read the response.
        let mut response = [0; usb::RESPONSE_SIZE];
        read_response(&mut handle, &mut response)?;
        close_device(handle, interfaces)?;

        // Check response.
        if crc16(&response[..6]) != CRC_RESIDUAL_OK {
            return Err(ChallengeResponseError::WrongCRC);
        }

        let serial = structure!("2I").unpack(response[..8].to_vec())?;

        Ok(serial.0)
    }

    pub fn challenge_response_hmac(&mut self, chall: &[u8], conf: Config) -> Result<Hmac> {
        let mut hmac = Hmac([0; 20]);

        let (mut handle, interfaces) =
            open_device(&mut self.context, conf.device.bus_id, conf.device.address_id)?;

        let mut challenge = [0; CHALLENGE_SIZE];

        if conf.variable && chall.last() == Some(&0) {
            challenge = [0xff; CHALLENGE_SIZE];
        }

        let mut command = Command::ChallengeHmac1;
        if let Slot::Slot2 = conf.slot {
            command = Command::ChallengeHmac2;
        }

        (&mut challenge[..chall.len()]).copy_from_slice(chall);
        let d = Frame::new(challenge, command);
        let mut buf = [0; usb::STATUS_UPDATE_PAYLOAD_SIZE];
        wait(
            &mut handle,
            |f| !f.contains(usb::Flags::SLOT_WRITE_FLAG),
            &mut buf,
        )?;

        write_frame(&mut handle, &d)?;

        // Read the response.
        let mut response = [0; usb::RESPONSE_SIZE];
        read_response(&mut handle, &mut response)?;
        close_device(handle, interfaces)?;

        // Check response.
        if crc16(&response[..22]) != CRC_RESIDUAL_OK {
            return Err(ChallengeResponseError::WrongCRC);
        }

        hmac.0.clone_from_slice(&response[..20]);

        Ok(hmac)
    }

    pub fn challenge_response_otp(&mut self, chall: &[u8], conf: Config) -> Result<Aes128Block> {
        let mut block = Aes128Block {
            block: GenericArray::clone_from_slice(&[0; 16]),
        };

        let (mut handle, interfaces) =
            open_device(&mut self.context, conf.device.bus_id, conf.device.address_id)?;

        let mut challenge = [0; CHALLENGE_SIZE];

        let mut command = Command::ChallengeOtp1;
        if let Slot::Slot2 = conf.slot {
            command = Command::ChallengeOtp2;
        }

        (&mut challenge[..chall.len()]).copy_from_slice(chall);
        let d = Frame::new(challenge, command);
        let mut buf = [0; usb::STATUS_UPDATE_PAYLOAD_SIZE];

        wait(
            &mut handle,
            |f| !f.contains(usb::Flags::SLOT_WRITE_FLAG),
            &mut buf,
        )?;

        write_frame(&mut handle, &d)?;

        let mut response = [0; usb::RESPONSE_SIZE];
        read_response(&mut handle, &mut response)?;
        close_device(handle, interfaces)?;

        // Check response.
        if crc16(&response[..18]) != CRC_RESIDUAL_OK {
            return Err(ChallengeResponseError::WrongCRC);
        }

        block.block.copy_from_slice(&response[..16]);

        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_device() {
        let mut cr_client = match ChallengeResponse::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{:?}", e);
                return;
            }
        };

        if let Err(e) = cr_client.find_device() {
            assert!(matches!(e, ChallengeResponseError::DeviceNotFound));
        };
    }

    #[test]
    fn test_find_all_devices() {
        let mut cr_client = match ChallengeResponse::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{:?}", e);
                return;
            }
        };

        if let Err(e) = cr_client.find_all_devices() {
            assert!(matches!(e, ChallengeResponseError::DeviceNotFound));
        };
    }
}
