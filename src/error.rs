#[cfg(feature = "rusb")]
use rusb::Error as usbError;
use std::error;
use std::fmt;
use std::io::Error as ioError;

#[derive(Debug)]
pub enum ChallengeResponseError {
    IOError(ioError),
    #[cfg(feature = "rusb")]
    UsbError(usbError),
    CommandNotSupported,
    DeviceNotFound,
    OpenDeviceError,
    CanNotWriteToDevice,
    CanNotReadFromDevice,
    WrongCRC,
    ConfigNotWritten,
    ListDevicesError,
}

impl fmt::Display for ChallengeResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ChallengeResponseError::IOError(ref err) => write!(f, "IO error: {}", err),
            #[cfg(feature = "rusb")]
            ChallengeResponseError::UsbError(ref err) => write!(f, "USB  error: {}", err),
            ChallengeResponseError::DeviceNotFound => write!(f, "Device not found"),
            ChallengeResponseError::OpenDeviceError => write!(f, "Can not open device"),
            ChallengeResponseError::CommandNotSupported => write!(f, "Command Not Supported"),
            ChallengeResponseError::WrongCRC => write!(f, "Wrong CRC"),
            ChallengeResponseError::CanNotWriteToDevice => write!(f, "Can not write to Device"),
            ChallengeResponseError::CanNotReadFromDevice => write!(f, "Can not read from Device"),
            ChallengeResponseError::ConfigNotWritten => write!(f, "Configuration has failed"),
            ChallengeResponseError::ListDevicesError => write!(f, "Could not list available devices"),
        }
    }
}

impl error::Error for ChallengeResponseError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            #[cfg(feature = "rusb")]
            ChallengeResponseError::UsbError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<ioError> for ChallengeResponseError {
    fn from(err: ioError) -> ChallengeResponseError {
        ChallengeResponseError::IOError(err)
    }
}

#[cfg(feature = "rusb")]
impl From<usbError> for ChallengeResponseError {
    fn from(err: usbError) -> ChallengeResponseError {
        ChallengeResponseError::UsbError(err)
    }
}
