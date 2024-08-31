use rusb::{Context as RUSBContext, DeviceHandle as RUSBDeviceHandle};
use std::time::Duration;
use std::{slice, thread};

use config::Command;
use error::ChallengeResponseError;
use sec::crc16;

mod rusb;
use usb::rusb::{raw_write, read};

pub use usb::rusb::{close_device, open_device};

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

pub type Context = RUSBContext;

pub(crate) type DeviceHandle = RUSBDeviceHandle<Context>;

pub fn write_frame(handle: &mut DeviceHandle, frame: &Frame) -> Result<(), ChallengeResponseError> {
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

pub fn wait<F: Fn(Flags) -> bool>(
    handle: &mut DeviceHandle,
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

/// Reset the write state after a read.
pub fn write_reset(handle: &mut DeviceHandle) -> Result<(), ChallengeResponseError> {
    raw_write(handle, &WRITE_RESET_PAYLOAD)?;
    let mut buf = [0; 8];
    wait(handle, |x| !x.contains(Flags::SLOT_WRITE_FLAG), &mut buf)?;
    Ok(())
}

pub fn read_response(handle: &mut DeviceHandle, response: &mut [u8]) -> Result<usize, ChallengeResponseError> {
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
