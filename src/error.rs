use std::io;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{} at offset {} (0x{1:X}); measured duration up to here: {:?}",
        .kind, .offset, .at_duration)]
pub struct MP3DurationError {
    #[source]
    pub kind: ErrorKind,
    pub offset: usize,
    pub at_duration: Duration,
}

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Invalid MPEG version")]
    ForbiddenVersion,
    #[error("Invalid MPEG Layer (0)")]
    ForbiddenLayer,
    #[error("Invalid bitrate bits: {0} (0b{0:b})", .bitrate)]
    InvalidBitrate { bitrate: u8 },
    #[error("Invalid sampling rate bits: {0} (0b{0:b})", .sampling_rate)]
    InvalidSamplingRate { sampling_rate: u8 },
    #[error("Unexpected frame, header 0x{:X}", .header)]
    UnexpectedFrame { header: u32 },
    #[error("Unexpected end of file")]
    UnexpectedEOF,
    #[error("MPEG frame too short")]
    MPEGFrameTooShort,
    #[error("Unexpected IO Error: {0}")]
    IOError(#[source] io::Error),
}

impl From<io::Error> for ErrorKind {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::UnexpectedEof => ErrorKind::UnexpectedEOF,
            _ => ErrorKind::IOError(e),
        }
    }
}
