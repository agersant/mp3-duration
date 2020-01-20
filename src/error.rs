use core::fmt;
use failure::Fail;
use std::io;
use std::time::Duration;

#[derive(Debug)]
pub struct MP3DurationError {
	pub kind: ErrorKind,
	pub offset: usize,
	pub at_duration: Duration,
}

impl fmt::Display for MP3DurationError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{} at offset {} (0x{1:X}); measured duration up to here: {:?}",
			self.kind, self.offset, self.at_duration
		)
	}
}

impl Fail for MP3DurationError {
	// Delegate `cause` to ErrorKind
	fn cause(&self) -> Option<&dyn Fail> {
		self.kind.cause()
	}
}

#[derive(Debug, Fail)]
pub enum ErrorKind {
	#[fail(display = "Invalid MPEG version")]
	ForbiddenVersion,
	#[fail(display = "Invalid MPEG Layer (0)")]
	ForbiddenLayer,
	#[fail(display = "Invalid bitrate bits: {0} (0b{0:b})", bitrate)]
	InvalidBitrate { bitrate: u8 },
	#[fail(display = "Invalid sampling rate bits: {0} (0b{0:b})", sampling_rate)]
	InvalidSamplingRate { sampling_rate: u8 },
	#[fail(display = "Unexpected frame, header 0x{:X}", header)]
	UnexpectedFrame { header: u32 },
	#[fail(display = "Unexpected end of file")]
	UnexpectedEOF,
	#[fail(display = "MPEG frame too short")]
	MPEGFrameTooShort,
	#[fail(display = "Unexpected IO Error: {}", _0)]
	IOError(#[fail(cause)] io::Error),
}

impl From<io::Error> for ErrorKind {
	fn from(e: io::Error) -> Self {
		match e.kind() {
			io::ErrorKind::UnexpectedEof => ErrorKind::UnexpectedEOF,
			_ => ErrorKind::IOError(e),
		}
	}
}
