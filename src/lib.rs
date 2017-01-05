use std::error::Error as StdError;
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::Path;
use std::time::Duration;

pub use BaudRate::*;

/// A module that exports traits that are useful to have in scope.
///
/// It is intended to be glob imported:
///
/// ```no_run
/// use serial::prelude::*;
/// ```
pub mod prelude {
    pub use ::{BaudRate, DataBits, FlowControl, Parity, StopBits};
    pub use ::{SerialPort, SerialPortInfo};
}

#[cfg(unix)]
pub mod posix;

#[cfg(windows)]
pub mod windows;

/// A type for results generated by interacting with serial ports.
///
/// The `Err` type is hard-wired to [`serial::Error`](struct.Error.html).
pub type Result<T> = std::result::Result<T,::Error>;

/// Categories of errors that can occur when interacting with serial ports.
///
/// This list is intended to grow over time and it is not recommended to exhaustively match against it.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ErrorKind {
    /// The device is not available.
    ///
    /// This could indicate that the device is in use by another process or was disconnected while
    /// performing I/O.
    NoDevice,

    /// A parameter was incorrect.
    InvalidInput,

    /// An unknown error occurred.
    Unknown,

    /// An I/O error occurred.
    ///
    /// The type of I/O error is determined by the inner `io::ErrorKind`.
    Io(io::ErrorKind)
}

/// An error type for serial port operations.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: String
}

impl Error {
    pub fn new<T: Into<String>>(kind: ErrorKind, description: T) -> Self {
        Error {
            kind: kind,
            description: description.into()
        }
    }

    /// Returns the corresponding `ErrorKind` for this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        fmt.write_str(&self.description)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Error {
        Error::new(ErrorKind::Io(io_error.kind()), format!("{}", io_error))
    }
}

impl From<Error> for io::Error {
    fn from(error: Error) -> io::Error {
        let kind = match error.kind {
            ErrorKind::NoDevice => io::ErrorKind::NotFound,
            ErrorKind::InvalidInput => io::ErrorKind::InvalidInput,
            ErrorKind::Unknown => io::ErrorKind::Other,
            ErrorKind::Io(kind) => kind
        };

        io::Error::new(kind, error.description)
    }
}

/// Serial port baud rates.
///
/// ## Portability
///
/// The `BaudRate` variants with numeric suffixes, e.g., `Baud9600`, indicate standard baud rates
/// that are widely-supported on many systems. While non-standard baud rates can be set with
/// `BaudOther`, their behavior is system-dependent. Some systems may not support arbitrary baud
/// rates. Using the standard baud rates is more likely to result in portable applications.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum BaudRate {
    /** 110 baud. */     Baud110,
    /** 300 baud. */     Baud300,
    /** 600 baud. */     Baud600,
    /** 1200 baud. */    Baud1200,
    /** 2400 baud. */    Baud2400,
    /** 4800 baud. */    Baud4800,
    /** 9600 baud. */    Baud9600,
    /** 19,200 baud. */  Baud19200,
    /** 38,400 baud. */  Baud38400,
    /** 57,600 baud. */  Baud57600,
    /** 115,200 baud. */ Baud115200,

    /// Non-standard baud rates.
    ///
    /// `BaudOther` can be used to set non-standard baud rates by setting its member to be the
    /// desired baud rate.
    ///
    /// ```no_run
    /// serial::BaudOther(4_000_000); // 4,000,000 baud
    /// ```
    ///
    /// Non-standard baud rates may not be supported on all systems.
    BaudOther(usize)
}

impl BaudRate {
    /// Creates a `BaudRate` for a particular speed.
    ///
    /// This function can be used to select a `BaudRate` variant from an integer containing the
    /// desired baud rate.
    ///
    /// ## Example
    ///
    /// ```
    /// # use serial::BaudRate;
    /// assert_eq!(BaudRate::Baud9600, BaudRate::from_speed(9600));
    /// assert_eq!(BaudRate::Baud115200, BaudRate::from_speed(115200));
    /// assert_eq!(BaudRate::BaudOther(4000000), BaudRate::from_speed(4000000));
    /// ```
    pub fn from_speed(speed: usize) -> BaudRate {
        match speed {
            110    => BaudRate::Baud110,
            300    => BaudRate::Baud300,
            600    => BaudRate::Baud600,
            1200   => BaudRate::Baud1200,
            2400   => BaudRate::Baud2400,
            4800   => BaudRate::Baud4800,
            9600   => BaudRate::Baud9600,
            19200  => BaudRate::Baud19200,
            38400  => BaudRate::Baud38400,
            57600  => BaudRate::Baud57600,
            115200 => BaudRate::Baud115200,
            n      => BaudRate::BaudOther(n),
        }
    }

    /// Returns the baud rate as an integer.
    ///
    /// ## Example
    ///
    /// ```
    /// # use serial::BaudRate;
    /// assert_eq!(9600, BaudRate::Baud9600.speed());
    /// assert_eq!(115200, BaudRate::Baud115200.speed());
    /// assert_eq!(4000000, BaudRate::BaudOther(4000000).speed());
    /// ```
    pub fn speed(&self) -> usize {
        match *self {
            BaudRate::Baud110      => 110,
            BaudRate::Baud300      => 300,
            BaudRate::Baud600      => 600,
            BaudRate::Baud1200     => 1200,
            BaudRate::Baud2400     => 2400,
            BaudRate::Baud4800     => 4800,
            BaudRate::Baud9600     => 9600,
            BaudRate::Baud19200    => 19200,
            BaudRate::Baud38400    => 38400,
            BaudRate::Baud57600    => 57600,
            BaudRate::Baud115200   => 115200,
            BaudRate::BaudOther(n) => n,
        }
    }
}

/// Number of bits per character.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum DataBits {
    // 5 bits per character
    Five,

    // 6 bits per character
    Six,

    // 7 bits per character
    Seven,

    // 8 bits per character
    Eight
}

/// Parity checking modes.
///
/// When parity checking is enabled (`Odd` or `Even`) an extra bit is transmitted with
/// each character. The value of the parity bit is arranged so that the number of 1 bits in the
/// character (including the parity bit) is an even number (`Even`) or an odd number
/// (`Odd`).
///
/// Parity checking is disabled by setting `None`, in which case parity bits are not
/// transmitted.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum Parity {
    /// No parity bit.
    None,

    /// Parity bit sets odd number of 1 bits.
    Odd,

    /// Parity bit sets even number of 1 bits.
    Even
}

/// Number of stop bits.
///
/// Stop bits are transmitted after every character.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum StopBits {
    /// One stop bit.
    One,

    /// Two stop bits.
    Two
}

/// Flow control modes.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum FlowControl {
    /// No flow control.
    None,

    /// Flow control using XON/XOFF bytes.
    Software,

    /// Flow control using RTS/CTS signals.
    Hardware
}


pub trait SerialPort: io::Read+io::Write {

    // Port settings getters
    fn baud_rate(&self) -> Option<BaudRate>;
    fn data_bits(&self) -> Option<DataBits>;
    fn flow_control(&self) -> Option<FlowControl>;
    fn parity(&self) -> Option<Parity>;
    fn stop_bits(&self) -> Option<StopBits>;
    fn timeout(&self) -> Duration;

    // Port settings setters
    fn set_baud_rate(&mut self, baud_rate: BaudRate) -> ::Result<()>;
    fn set_data_bits(&mut self, data_bits: DataBits) -> ::Result<()>;
    fn set_flow_control(&mut self, flow_control: FlowControl) -> ::Result<()>;
    fn set_parity(&mut self, parity: Parity) -> ::Result<()>;
    fn set_stop_bits(&mut self, stop_bits: StopBits) -> ::Result<()>;
    fn set_timeout(&mut self, timeout: Duration) -> ::Result<()>;

    // Functions for setting additional pins
    fn write_request_to_send(&mut self, level: bool) -> ::Result<()>;
    fn write_data_terminal_ready(&mut self, level: bool) -> ::Result<()>;

    // Functions for reading additional pins
    fn read_clear_to_send(&mut self) -> ::Result<bool>;
    fn read_data_set_ready(&mut self) -> ::Result<bool>;
    fn read_ring_indicator(&mut self) -> ::Result<bool>;
    fn read_carrier_data(&mut self) -> ::Result<bool>;
}

/// A device-independent implementation of serial port information.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SerialPortInfo {
    /// Port name
    pub port_name: String,
}

pub fn open<T: AsRef<OsStr> + ?Sized>(port: &T) -> ::Result<Box<SerialPort>> {
    // This is written with explicit returns because of:
    // https://github.com/rust-lang/rust/issues/38337

    #[cfg(unix)]
    return posix::TTYPort::open(Path::new(port));

    #[cfg(windows)]
    return posix::COMPort::open(Path::new(port));

    #[cfg(not(any(unix, windows)))]
    Err(Error::new(ErrorKind::Unknown, "open() not implemented for platform"))
}

pub fn available_ports() -> ::Result<Vec<SerialPortInfo>> {
    #[cfg(unix)]
    return posix::available_ports();

    #[cfg(windows)]
    return windows::available_ports();

    #[cfg(not(any(unix, windows)))]
    Err(Error::new(ErrorKind::Unknown, "available_ports() not implemented for platform"))
}

pub fn available_baud_rates() -> Vec<u32> {
    #[cfg(unix)]
    return posix::available_baud_rates();

    #[cfg(windows)]
    return windows::available_baud_rates();

    #[cfg(not(any(unix, windows)))]
    return Vec::new();
}
