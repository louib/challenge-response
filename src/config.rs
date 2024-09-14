use std::fmt::Display;

use crate::Device;

#[derive(Clone, Debug, PartialEq)]
pub enum Slot {
    Slot1,
    Slot2,
}

impl Slot {
    /// Parses a slot number from a slice.
    /// Returns None if the slot number is invalid.
    pub fn from_str(slot_number: &str) -> Option<Slot> {
        if slot_number == "1" {
            return Some(Slot::Slot1);
        }
        if slot_number == "2" {
            return Some(Slot::Slot2);
        }
        None
    }

    /// Parses a slot number from an integer.
    /// Returns None if the slot number is invalid.
    pub fn from_int(slot_number: usize) -> Option<Slot> {
        if slot_number == 1 {
            return Some(Slot::Slot1);
        }
        if slot_number == 2 {
            return Some(Slot::Slot2);
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Sha1,
    Otp,
}

/// From the Validation Protocol documentation:
///
/// A value 0 to 100 indicating percentage of syncing required by client,
/// or strings "fast" or "secure" to use server-configured values; if
/// absent, let the server decide.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SyncLevel(u8);

impl SyncLevel {
    pub fn fast() -> SyncLevel {
        SyncLevel(0)
    }

    pub fn secure() -> SyncLevel {
        SyncLevel(100)
    }

    pub fn custom(level: u8) -> SyncLevel {
        if level > 100 {
            SyncLevel(100)
        } else {
            SyncLevel(level)
        }
    }
}

impl Display for SyncLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Command {
    Configuration1 = 0x01,
    Configuration2 = 0x03,
    Update1 = 0x04,
    Update2 = 0x05,
    Swap = 0x06,
    DeviceSerial = 0x10,
    DeviceConfig = 0x11,
    ChallengeOtp1 = 0x20,
    ChallengeOtp2 = 0x28,
    ChallengeHmac1 = 0x30,
    ChallengeHmac2 = 0x38,
    ReadConfig1 = 0x1c,
    ReadConfig2 = 0x1d,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub device: Device,
    pub variable: bool,
    pub slot: Slot,
    pub mode: Mode,
    pub command: Command,
}

impl Config {
    pub fn new_from(device: Device) -> Config {
        Config {
            device,
            variable: true,
            slot: Slot::Slot2,
            mode: Mode::Sha1,
            command: Command::ChallengeHmac2,
        }
    }

    pub fn set_variable_size(mut self, variable: bool) -> Self {
        self.variable = variable;
        self
    }

    pub fn set_slot(mut self, slot: Slot) -> Self {
        self.slot = slot;
        self
    }

    pub fn set_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set_command(mut self, command: Command) -> Self {
        self.command = command;
        self
    }
}
