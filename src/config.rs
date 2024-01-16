use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Slot {
    Slot1,
    Slot2,
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub product_id: u16,
    pub vendor_id: u16,
    pub variable: bool,
    pub slot: Slot,
    pub mode: Mode,
    pub command: Command,
}

#[allow(dead_code)]
impl Config {
    pub fn default() -> Config {
        Config {
            product_id: 0x00,
            vendor_id: 0x1050,
            variable: true,
            slot: Slot::Slot1,
            mode: Mode::Sha1,
            command: Command::ChallengeHmac1,
        }
    }

    pub fn set_vendor_id(mut self, vendor_id: u16) -> Self {
        self.vendor_id = vendor_id;
        self
    }

    pub fn set_product_id(mut self, product_id: u16) -> Self {
        self.product_id = product_id;
        self
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
