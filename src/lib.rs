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
pub mod hmacmode;
mod manager;
pub mod otpmode;
pub mod sec;
pub mod yubicoerror;

use aes::cipher::generic_array::GenericArray;

use config::Command;
use config::{Config, Slot};
use configure::DeviceModeConfig;
use hmacmode::Hmac;
use manager::{Flags, Frame};
use otpmode::Aes128Block;
use rusb::{Context, UsbContext};
use sec::{crc16, CRC_RESIDUAL_OK};
use yubicoerror::YubicoError;

const VENDOR_ID: u16 = 0x1050;

/// The `Result` type used in this crate.
type Result<T> = ::std::result::Result<T, YubicoError>;

#[derive(Clone, Debug, PartialEq)]
pub struct Yubikey {
    pub name: Option<String>,
    pub serial: Option<u32>,
    pub product_id: u16,
    pub vendor_id: u16,
    pub bus_id: u8,
    pub address_id: u8,
}

pub struct Yubico {
    context: Context,
}

impl Yubico {
    /// Creates a new Yubico instance.
    pub fn new() -> Self {
        Yubico {
            context: Context::new().unwrap(),
        }
    }

    fn read_serial_from_device(&mut self, device: rusb::Device<Context>) -> Result<u32> {
        let mut handle = manager::open_device(&mut self.context, device.bus_number(), device.address())?;
        let challenge = [0; 64];
        let command = Command::DeviceSerial;

        let d = Frame::new(challenge, command); // FixMe: do not need a challange
        let mut buf = [0; 8];
        manager::wait(&mut handle.0, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;

        manager::write_frame(&mut handle.0, &d)?;

        // Read the response.
        let mut response = [0; 36];
        manager::read_response(&mut handle.0, &mut response)?;

        drop(handle); // Close Device

        // Check response.
        if crc16(&response[..6]) != crate::sec::CRC_RESIDUAL_OK {
            return Err(YubicoError::WrongCRC);
        }

        let serial = structure!("2I").unpack(response[..8].to_vec())?;

        Ok(serial.0)
    }

    pub fn find_yubikey(&mut self) -> Result<Yubikey> {
        for device in self.context.devices().unwrap().iter() {
            let descr = device.device_descriptor().unwrap();
            if descr.vendor_id() == VENDOR_ID {
                let name = device.open()?.read_product_string_ascii(&descr).ok();
                let serial = self.read_serial_from_device(device.clone()).ok();
                let yubikey = Yubikey {
                    name: name,
                    serial: serial,
                    product_id: descr.product_id(),
                    vendor_id: descr.vendor_id(),
                    bus_id: device.bus_number(),
                    address_id: device.address(),
                };

                return Ok(yubikey);
            }
        }

        Err(YubicoError::DeviceNotFound)
    }

    pub fn find_yubikey_from_serial(&mut self, serial: u32) -> Result<Yubikey> {
        for device in self.context.devices().unwrap().iter() {
            let descr = device.device_descriptor().unwrap();
            if descr.vendor_id() == VENDOR_ID {
                let name = device.open()?.read_product_string_ascii(&descr).ok();
                let fetched_serial = match self.read_serial_from_device(device.clone()).ok() {
                    Some(s) => s,
                    None => 0,
                };
                if serial == fetched_serial {
                    let yubikey = Yubikey {
                        name: name,
                        serial: Some(serial),
                        product_id: descr.product_id(),
                        vendor_id: descr.vendor_id(),
                        bus_id: device.bus_number(),
                        address_id: device.address(),
                    };

                    return Ok(yubikey);
                }
            }
        }

        Err(YubicoError::DeviceNotFound)
    }

    pub fn find_all_yubikeys(&mut self) -> Result<Vec<Yubikey>> {
        let mut result: Vec<Yubikey> = Vec::new();
        for device in self.context.devices().unwrap().iter() {
            let descr = device.device_descriptor().unwrap();
            if descr.vendor_id() == VENDOR_ID {
                let name = device.open()?.read_product_string_ascii(&descr).ok();
                let serial = self.read_serial_from_device(device.clone()).ok();
                let yubikey = Yubikey {
                    name: name,
                    serial: serial,
                    product_id: descr.product_id(),
                    vendor_id: descr.vendor_id(),
                    bus_id: device.bus_number(),
                    address_id: device.address(),
                };
                result.push(yubikey);
            }
        }

        if !result.is_empty() {
            return Ok(result);
        }

        Err(YubicoError::DeviceNotFound)
    }

    pub fn write_config(&mut self, conf: Config, device_config: &mut DeviceModeConfig) -> Result<()> {
        let d = device_config.to_frame(conf.command);
        let mut buf = [0; 8];

        match manager::open_device(&mut self.context, conf.yubikey.bus_id, conf.yubikey.address_id) {
            Ok((mut handle, interfaces)) => {
                manager::wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;

                // TODO: Should check version number.

                manager::write_frame(&mut handle, &d)?;
                manager::wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
                manager::close_device(handle, interfaces)?;

                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn read_serial_number(&mut self, conf: Config) -> Result<u32> {
        match manager::open_device(&mut self.context, conf.yubikey.bus_id, conf.yubikey.address_id) {
            Ok((mut handle, interfaces)) => {
                let challenge = [0; 64];
                let command = Command::DeviceSerial;

                let d = Frame::new(challenge, command); // FixMe: do not need a challange
                let mut buf = [0; 8];
                manager::wait(
                    &mut handle,
                    |f| !f.contains(manager::Flags::SLOT_WRITE_FLAG),
                    &mut buf,
                )?;

                manager::write_frame(&mut handle, &d)?;

                // Read the response.
                let mut response = [0; 36];
                manager::read_response(&mut handle, &mut response)?;
                manager::close_device(handle, interfaces)?;

                // Check response.
                if crc16(&response[..6]) != CRC_RESIDUAL_OK {
                    return Err(YubicoError::WrongCRC);
                }

                let serial = structure!("2I").unpack(response[..8].to_vec())?;

                Ok(serial.0)
            }
            Err(error) => Err(error),
        }
    }

    pub fn challenge_response_hmac(&mut self, chall: &[u8], conf: Config) -> Result<Hmac> {
        let mut hmac = Hmac([0; 20]);

        match manager::open_device(&mut self.context, conf.yubikey.bus_id, conf.yubikey.address_id) {
            Ok((mut handle, interfaces)) => {
                let mut challenge = [0; 64];

                if conf.variable && chall.last() == Some(&0) {
                    challenge = [0xff; 64];
                }

                let mut command = Command::ChallengeHmac1;
                if let Slot::Slot2 = conf.slot {
                    command = Command::ChallengeHmac2;
                }

                (&mut challenge[..chall.len()]).copy_from_slice(chall);
                let d = Frame::new(challenge, command);
                let mut buf = [0; 8];
                manager::wait(
                    &mut handle,
                    |f| !f.contains(manager::Flags::SLOT_WRITE_FLAG),
                    &mut buf,
                )?;

                manager::write_frame(&mut handle, &d)?;

                // Read the response.
                let mut response = [0; 36];
                manager::read_response(&mut handle, &mut response)?;
                manager::close_device(handle, interfaces)?;

                // Check response.
                if crc16(&response[..22]) != CRC_RESIDUAL_OK {
                    return Err(YubicoError::WrongCRC);
                }

                hmac.0.clone_from_slice(&response[..20]);

                Ok(hmac)
            }
            Err(error) => Err(error),
        }
    }

    pub fn challenge_response_otp(&mut self, chall: &[u8], conf: Config) -> Result<Aes128Block> {
        let mut block = Aes128Block {
            block: GenericArray::clone_from_slice(&[0; 16]),
        };

        match manager::open_device(&mut self.context, conf.yubikey.bus_id, conf.yubikey.address_id) {
            Ok((mut handle, interfaces)) => {
                let mut challenge = [0; 64];
                //(&mut challenge[..6]).copy_from_slice(chall);

                let mut command = Command::ChallengeOtp1;
                if let Slot::Slot2 = conf.slot {
                    command = Command::ChallengeOtp2;
                }

                (&mut challenge[..chall.len()]).copy_from_slice(chall);
                let d = Frame::new(challenge, command);
                let mut buf = [0; 8];

                let mut response = [0; 36];
                manager::wait(
                    &mut handle,
                    |f| !f.contains(manager::Flags::SLOT_WRITE_FLAG),
                    &mut buf,
                )?;
                manager::write_frame(&mut handle, &d)?;
                manager::read_response(&mut handle, &mut response)?;
                manager::close_device(handle, interfaces)?;

                // Check response.
                if crc16(&response[..18]) != CRC_RESIDUAL_OK {
                    return Err(YubicoError::WrongCRC);
                }

                block.block.copy_from_slice(&response[..16]);

                Ok(block)
            }
            Err(error) => Err(error),
        }
    }
}
