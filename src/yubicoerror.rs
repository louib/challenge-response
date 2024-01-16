use rusb::Error as usbError;
use std::error;
use std::fmt;
use std::io::Error as ioError;

#[derive(Debug)]
pub enum YubicoError {
    IOError(ioError),
    UsbError(usbError),
    CommandNotSupported,
    DeviceNotFound,
    OpenDeviceError,
    CanNotWriteToDevice,
    WrongCRC,
    ConfigNotWritten,
}

impl fmt::Display for YubicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            YubicoError::IOError(ref err) => write!(f, "IO error: {}", err),
            YubicoError::UsbError(ref err) => write!(f, "USB  error: {}", err),
            YubicoError::DeviceNotFound => write!(f, "Device not found"),
            YubicoError::OpenDeviceError => write!(f, "Can not open device"),
            YubicoError::CommandNotSupported => write!(f, "Command Not Supported"),
            YubicoError::WrongCRC => write!(f, "Wrong CRC"),
            YubicoError::CanNotWriteToDevice => write!(f, "Can not write to Device"),
            YubicoError::ConfigNotWritten => write!(f, "Configuration has failed"),
        }
    }
}

impl error::Error for YubicoError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            YubicoError::UsbError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<ioError> for YubicoError {
    fn from(err: ioError) -> YubicoError {
        YubicoError::IOError(err)
    }
}

impl From<usbError> for YubicoError {
    fn from(err: usbError) -> YubicoError {
        YubicoError::UsbError(err)
    }
}
