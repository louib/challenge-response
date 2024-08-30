use config::Command;
use sec::crc16;

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
