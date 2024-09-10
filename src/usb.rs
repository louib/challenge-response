use std::time::Duration;
use std::{slice, thread};

use config::Command;
use error::ChallengeResponseError;
use sec::crc16;

#[cfg(feature = "rusb")]
pub type BackendType = rusb::RUSBBackend;
#[cfg(all(feature = "nusb", not(feature = "rusb")))]
pub type BackendType = nusb::NUSBBackend;

/// If using a variable-length challenge, the challenge must be stricly smaller than this value.
/// If using a fixed-length challenge, the challenge must be exactly equal to this value.
pub const CHALLENGE_SIZE: usize = 64;

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

#[cfg(all(feature = "nusb", not(feature = "rusb")))]
pub mod nusb;
#[cfg(feature = "rusb")]
pub mod rusb;

/// The size of the payload when writing a request to the usb interface.
pub(crate) const PAYLOAD_SIZE: usize = 64;
/// The size of the response after writing a request to the usb interface.
pub(crate) const RESPONSE_SIZE: usize = 36;
/// The size of the payload to change the state of the device
pub(crate) const STATUS_UPDATE_PAYLOAD_SIZE: usize = 8;

pub(crate) const HID_GET_REPORT: u8 = 0x01;
pub(crate) const HID_SET_REPORT: u8 = 0x09;
pub(crate) const REPORT_TYPE_FEATURE: u16 = 0x03;

pub(crate) const WRITE_RESET_PAYLOAD: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0x8f];

bitflags! {
    pub struct Flags: u8 {
        const SLOT_WRITE_FLAG = 0x80;
        const RESP_PENDING_FLAG = 0x40;
    }
}

#[repr(C)]
#[repr(packed)]
pub struct Frame {
    pub payload: [u8; PAYLOAD_SIZE],
    command: Command,
    crc: u16,
    filler: [u8; 3],
}

impl Frame {
    pub fn new(payload: [u8; PAYLOAD_SIZE], command: Command) -> Self {
        let mut f = Frame {
            payload,
            command,
            crc: 0,
            filler: [0; 3],
        };
        f.crc = crc16(&f.payload).to_le();
        f
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub name: Option<String>,
    pub serial: Option<u32>,
    pub product_id: u16,
    pub vendor_id: u16,
    pub bus_id: u8,
    pub address_id: u8,
}

pub trait Backend<DeviceHandle, Interface> {
    fn new() -> Result<Self, ChallengeResponseError>
    where
        Self: Sized;

    fn open_device(
        &mut self,
        bus_id: u8,
        address_id: u8,
    ) -> Result<(DeviceHandle, Vec<Interface>), ChallengeResponseError>;

    fn close_device(
        &self,
        handle: DeviceHandle,
        interfaces: Vec<Interface>,
    ) -> Result<(), ChallengeResponseError>;

    fn read(&self, handle: &mut DeviceHandle, buf: &mut [u8]) -> Result<usize, ChallengeResponseError>;
    fn raw_write(&self, handle: &mut DeviceHandle, packet: &[u8]) -> Result<(), ChallengeResponseError>;

    fn find_device(&mut self) -> Result<Device, ChallengeResponseError>;
    fn find_device_from_serial(&mut self, serial: u32) -> Result<Device, ChallengeResponseError>;
    fn find_all_devices(&mut self) -> Result<Vec<Device>, ChallengeResponseError>;

    fn write_frame(&self, handle: &mut DeviceHandle, frame: &Frame) -> Result<(), ChallengeResponseError> {
        let mut data = unsafe { slice::from_raw_parts(frame as *const Frame as *const u8, 70) };

        let mut seq = 0;
        let mut buf = [0; 8];
        while !data.is_empty() {
            let (a, b) = data.split_at(7);

            if seq == 0 || b.is_empty() || a.iter().any(|&x| x != 0) {
                let mut packet = [0; 8];
                (&mut packet[..7]).copy_from_slice(a);

                packet[7] = Flags::SLOT_WRITE_FLAG.bits() + seq;
                self.wait(handle, |x| !x.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
                self.raw_write(handle, &packet)?;
            }
            data = b;
            seq += 1
        }
        Ok(())
    }

    fn wait<F: Fn(Flags) -> bool>(
        &self,
        handle: &mut DeviceHandle,
        f: F,
        buf: &mut [u8],
    ) -> Result<(), ChallengeResponseError> {
        loop {
            self.read(handle, buf)?;
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

    /// Reset the write state after a read.
    fn write_reset(&self, handle: &mut DeviceHandle) -> Result<(), ChallengeResponseError> {
        self.raw_write(handle, &WRITE_RESET_PAYLOAD)?;
        let mut buf = [0; 8];
        self.wait(handle, |x| !x.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
        Ok(())
    }

    fn read_response(
        &self,
        handle: &mut DeviceHandle,
        response: &mut [u8],
    ) -> Result<usize, ChallengeResponseError> {
        let mut r0 = 0;
        self.wait(
            handle,
            |f| f.contains(Flags::RESP_PENDING_FLAG),
            &mut response[..8],
        )?;
        r0 += 7;
        loop {
            if self.read(handle, &mut response[r0..r0 + 8])? < 8 {
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
        self.write_reset(handle)?;
        Ok(r0)
    }

    fn read_serial_from_device(
        &mut self,
        device_bus_id: u8,
        device_address: u8,
    ) -> Result<u32, ChallengeResponseError> {
        let (mut handle, interfaces) = self.open_device(device_bus_id, device_address)?;
        let challenge = [0; CHALLENGE_SIZE];
        let command = Command::DeviceSerial;

        let d = Frame::new(challenge, command); // FIXME: do not need a challange
        let mut buf = [0; STATUS_UPDATE_PAYLOAD_SIZE];
        self.wait(&mut handle, |f| !f.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;

        self.write_frame(&mut handle, &d)?;

        // Read the response.
        let mut response = [0; RESPONSE_SIZE];
        self.read_response(&mut handle, &mut response)?;
        self.close_device(handle, interfaces)?;

        // Check response.
        if crc16(&response[..6]) != crate::sec::CRC_RESIDUAL_OK {
            return Err(ChallengeResponseError::WrongCRC);
        }

        let serial = structure!("2I").unpack(response[..8].to_vec())?;

        Ok(serial.0)
    }
}
