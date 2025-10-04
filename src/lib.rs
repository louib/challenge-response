#![doc = include_str!("../README.md")]

#[cfg(not(any(feature = "rusb", feature = "nusb")))]
compile_error!("Either the rusb or nusb feature must be enabled for this crate");

#[cfg(all(feature = "nusb", not(feature = "rusb"), not(target_os = "windows")))]
extern crate nusb;
#[cfg(any(feature = "rusb", target_os = "windows"))]
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
use sec::{crc16, CRC_RESIDUAL_OK};
use usb::{Backend, BackendType, Flags, Frame, Status, CHALLENGE_SIZE};

pub use usb::Device;

/// The `Result` type used in this crate.
type Result<T> = ::std::result::Result<T, ChallengeResponseError>;

pub struct ChallengeResponse {
    backend: BackendType,
}

impl ChallengeResponse {
    /// Creates a new ChallengeResponse instance.
    pub fn new() -> Result<Self> {
        let backend = BackendType::new()?;
        Ok(ChallengeResponse { backend })
    }

    pub fn find_device(&mut self) -> Result<Device> {
        self.backend.find_device()
    }

    pub fn find_device_from_serial(&mut self, serial: u32) -> Result<Device> {
        self.backend.find_device_from_serial(serial)
    }

    pub fn find_all_devices(&mut self) -> Result<Vec<Device>> {
        self.backend.find_all_devices()
    }

    pub fn read_serial_number(&mut self, conf: Config) -> Result<u32> {
        self.backend
            .read_serial_from_device(conf.device.bus_id, conf.device.address_id)
    }

    pub fn read_status(&mut self, conf: Config) -> Result<Status> {
        let (mut handle, interfaces) = self
            .backend
            .open_device(conf.device.bus_id, conf.device.address_id)?;

        let challenge = [0; CHALLENGE_SIZE];
        let command = Command::DeviceConfig;

        let d = Frame::new(challenge, command);
        let mut buf = [0; usb::STATUS_UPDATE_PAYLOAD_SIZE];
        self.backend
            .wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
        self.backend.write_frame(&mut handle, &d)?;

        // Read the response.
        let mut response = [0; usb::RESPONSE_SIZE];
        self.backend.wait(
            &mut handle,
            |f| !f.contains(Flags::SLOT_WRITE_FLAG),
            &mut response[..8],
        )?;
        self.backend.write_reset(&mut handle)?;
        self.backend.close_device(handle, interfaces)?;

        // FIXME I can't get the CRC check to work here, assuming that it is needed.
        // Check response.
        // if crc16(&response[..6]) != CRC_RESIDUAL_OK {
        //     return Err(ChallengeResponseError::WrongCRC);
        // }

        let slice = &response[..6];
        let array: [u8; 6] = slice.try_into().unwrap();
        let status: Status = unsafe { std::mem::transmute(array) };

        Ok(status)
    }

    pub fn is_configured(&mut self, device: Device, slot: Slot) -> Result<bool> {
        let conf = Config::new_from(device);
        let status = self.read_status(conf)?;

        if status.pgm_seq == 0 {
            return Ok(false);
        }

        let configured = match slot {
            Slot::Slot1 => (status.touch_level & 1) != 0,
            Slot::Slot2 => (status.touch_level & 2) != 0,
        };

        Ok(configured)
    }

    pub fn write_config(&mut self, conf: Config, device_config: &mut DeviceModeConfig) -> Result<()> {
        let d = device_config.to_frame(conf.command);
        let mut buf = [0; usb::STATUS_UPDATE_PAYLOAD_SIZE];

        let (mut handle, interfaces) = self
            .backend
            .open_device(conf.device.bus_id, conf.device.address_id)?;

        self.backend
            .wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;

        // TODO: Should check version number.

        self.backend.write_frame(&mut handle, &d)?;
        self.backend
            .wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
        self.backend.close_device(handle, interfaces)?;

        Ok(())
    }

    pub fn challenge_response_hmac(&mut self, chall: &[u8], conf: Config) -> Result<Hmac> {
        let mut hmac = Hmac([0; 20]);

        let (mut handle, interfaces) = self
            .backend
            .open_device(conf.device.bus_id, conf.device.address_id)?;

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
        self.backend.wait(
            &mut handle,
            |f| !f.contains(usb::Flags::SLOT_WRITE_FLAG),
            &mut buf,
        )?;

        self.backend.write_frame(&mut handle, &d)?;

        // Read the response.
        let mut response = [0; usb::RESPONSE_SIZE];
        self.backend.read_response(&mut handle, &mut response)?;
        self.backend.close_device(handle, interfaces)?;

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

        let (mut handle, interfaces) = self
            .backend
            .open_device(conf.device.bus_id, conf.device.address_id)?;

        let mut challenge = [0; CHALLENGE_SIZE];

        let mut command = Command::ChallengeOtp1;
        if let Slot::Slot2 = conf.slot {
            command = Command::ChallengeOtp2;
        }

        (&mut challenge[..chall.len()]).copy_from_slice(chall);
        let d = Frame::new(challenge, command);
        let mut buf = [0; usb::STATUS_UPDATE_PAYLOAD_SIZE];

        self.backend.wait(
            &mut handle,
            |f| !f.contains(usb::Flags::SLOT_WRITE_FLAG),
            &mut buf,
        )?;

        self.backend.write_frame(&mut handle, &d)?;

        let mut response = [0; usb::RESPONSE_SIZE];
        self.backend.read_response(&mut handle, &mut response)?;
        self.backend.close_device(handle, interfaces)?;

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
