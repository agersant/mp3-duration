extern crate failure;

use core::fmt;
use failure::Fail;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
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

macro_rules! bail {
    ($ctx:expr, $err:expr) => {
        return Err(MP3DurationError {
            kind: $err.into(),
            offset: $ctx.0,
            at_duration: $ctx.1,
        });
    };
}

#[derive(Clone, Copy, Debug)]
enum Version {
    Mpeg1,
    Mpeg2,
    Mpeg25,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Layer {
    NotDefined,
    Layer1,
    Layer2,
    Layer3,
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    Stereo,
    JointStereo,
    DualChannel,
    Mono,
}

static BIT_RATES: [[[u32; 16]; 4]; 3] = [
    [
        [0; 16],
        [
            // Mpeg1 Layer1
            0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448, 0,
        ],
        [
            // Mpeg1 Layer2
            0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0,
        ],
        [
            // Mpeg1 Layer3
            0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
        ],
    ],
    [
        [0; 16],
        [
            // Mpeg2 Layer1
            0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256, 0,
        ],
        [
            // Mpeg2 Layer2
            0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
        ],
        [
            // Mpeg2 Layer3
            0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
        ],
    ],
    [
        [0; 16],
        [
            // Mpeg25 Layer1
            0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256, 0,
        ],
        [
            // Mpeg25 Layer2
            0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
        ],
        [
            // Mpeg25 Layer3
            0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
        ],
    ],
];

static SAMPLING_RATES: [[u32; 4]; 3] = [
    [44100, 48000, 32000, 0], // Mpeg1
    [22050, 24000, 16000, 0], // Mpeg2
    [11025, 12000, 8000, 0],  // Mpeg25
];

static SAMPLES_PER_FRAME: [[u32; 4]; 3] = [
    [0, 384, 1152, 1152], // Mpeg1
    [0, 384, 1152, 576],  // Mpeg2
    [0, 384, 1152, 576],  // Mpeg25
];

static SIDE_INFORMATION_SIZES: [[u32; 4]; 3] = [
    [32, 32, 32, 17], // Mpeg1
    [17, 17, 17, 9],  // Mpeg2
    [17, 17, 17, 9],  // Mpeg25
];

fn get_bitrate(
    version: Version,
    layer: Layer,
    encoded_bitrate: u8,
    ctx: (usize, Duration),
) -> Result<u32, MP3DurationError> {
    if encoded_bitrate >= 15 {
        bail!(
            ctx,
            ErrorKind::InvalidBitrate {
                bitrate: encoded_bitrate
            }
        );
    }
    if layer == Layer::NotDefined {
        bail!(ctx, ErrorKind::ForbiddenLayer);
    }
    Ok(1000 * BIT_RATES[version as usize][layer as usize][encoded_bitrate as usize])
}

fn get_sampling_rate(
    version: Version,
    encoded_sampling_rate: u8,
    ctx: (usize, Duration),
) -> Result<u32, MP3DurationError> {
    if encoded_sampling_rate >= 3 {
        bail!(
            ctx,
            ErrorKind::InvalidSamplingRate {
                sampling_rate: encoded_sampling_rate
            }
        );
    }
    Ok(SAMPLING_RATES[version as usize][encoded_sampling_rate as usize])
}

fn get_samples_per_frame(
    version: Version,
    layer: Layer,
    ctx: (usize, Duration),
) -> Result<u32, MP3DurationError> {
    if layer == Layer::NotDefined {
        bail!(ctx, ErrorKind::ForbiddenLayer);
    }
    Ok(SAMPLES_PER_FRAME[version as usize][layer as usize])
}

fn get_side_information_size(version: Version, mode: Mode) -> usize {
    SIDE_INFORMATION_SIZES[version as usize][mode as usize] as usize
}

fn skip<T>(reader: &mut T, dump: &mut Vec<u8>, advance: usize) -> Result<(), std::io::Error>
where
    T: Read,
{
    dump.resize(advance, 0);
    reader.read_exact(&mut dump[..])
}

/// Measures the duration of a mp3 file contained in any struct implementing Read.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use std::fs::File;
/// use std::io::BufReader;
/// use mp3_duration;
///
/// let path = Path::new("test/source.mp3");
/// let file = File::open(path).unwrap();
/// let mut reader = BufReader::new(file);
/// let duration = mp3_duration::from_read(&mut reader).unwrap();
/// println!("File duration: {:?}", duration);
/// ```
pub fn from_read<T>(reader: &mut T) -> Result<Duration, MP3DurationError>
where
    T: Read,
{
    let mut header_buffer = [0; 4];
    let mut dump = vec![0; 16 * 1024];

    let mut bytes_read = 0;
    let mut duration = Duration::from_secs(0);
    loop {
        // Skip over all 0x00 bytes (these are probably incorrectly added padding bytes for id3v2)
        header_buffer[0] = 0;
        while header_buffer[0] == 0 {
            match reader.read_exact(&mut header_buffer[0..1]) {
                Ok(_) => (),
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => bail!((bytes_read, duration), e),
            };
            bytes_read += 1;
        }

        match reader.read_exact(&mut header_buffer[1..]) {
            Ok(_) => (),
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => bail!((bytes_read, duration), e),
        };
        bytes_read += 3;

        // MPEG frame
        let header = (header_buffer[0] as u32) << 24
            | (header_buffer[1] as u32) << 16
            | (header_buffer[2] as u32) << 8
            | header_buffer[3] as u32;
        let is_mp3 = header >> 21 == 0x7FF;
        if is_mp3 {
            let version = match (header >> 19) & 0b11 {
                0 => Version::Mpeg25,
                1 => bail!((bytes_read, duration), ErrorKind::ForbiddenVersion),
                2 => Version::Mpeg2,
                3 => Version::Mpeg1,
                _ => unreachable!(),
            };

            let layer = match (header >> 17) & 0b11 {
                0 => Layer::NotDefined,
                1 => Layer::Layer3,
                2 => Layer::Layer2,
                3 => Layer::Layer1,
                _ => unreachable!(),
            };

            let encoded_bitrate = (header >> 12) & 0b1111;
            let encoded_sampling_rate = (header >> 10) & 0b11;
            let padding = if 0 != ((header >> 9) & 1) { 1 } else { 0 };

            let mode = match (header >> 6) & 0b11 {
                0 => Mode::Stereo,
                1 => Mode::JointStereo,
                2 => Mode::DualChannel,
                3 => Mode::Mono,
                _ => unreachable!(),
            };

            let sampling_rate =
                get_sampling_rate(version, encoded_sampling_rate as u8, (bytes_read, duration))?;
            let num_samples = get_samples_per_frame(version, layer, (bytes_read, duration))?;

            let xing_offset = get_side_information_size(version, mode);
            let mut xing_buffer = [0; 12];
            dump.resize(xing_offset, 0);

            if let Err(e) = reader.read_exact(&mut dump[..xing_offset]) {
                bail!((bytes_read, duration), e);
            }

            if let Err(e) = reader.read_exact(&mut xing_buffer) {
                bail!((bytes_read + xing_offset, duration), e);
            }

            let is_xing = xing_buffer[0] == 'X' as u8
                && xing_buffer[1] == 'i' as u8
                && xing_buffer[2] == 'n' as u8
                && xing_buffer[3] == 'g' as u8;
            let is_info = xing_buffer[0] == 'I' as u8
                && xing_buffer[1] == 'n' as u8
                && xing_buffer[2] == 'f' as u8
                && xing_buffer[3] == 'o' as u8;
            if is_xing || is_info {
                let has_frames = 0 != (xing_buffer[7] & 1);
                if has_frames {
                    let num_frames = (xing_buffer[8] as u32) << 24
                        | (xing_buffer[9] as u32) << 16
                        | (xing_buffer[10] as u32) << 8
                        | xing_buffer[11] as u32;
                    let rate = sampling_rate as u64;
                    let billion = 1_000_000_000;
                    let frames_x_samples = num_frames as u64 * num_samples as u64;
                    let seconds = frames_x_samples / rate;
                    let nanoseconds = (billion * frames_x_samples) / rate - billion * seconds;
                    return Ok(Duration::new(seconds, nanoseconds as u32));
                }
            }

            let bitrate = get_bitrate(
                version,
                layer,
                encoded_bitrate as u8,
                (bytes_read, duration),
            )?;
            let frame_length = (num_samples / 8 * bitrate / sampling_rate + padding) as usize;

            let bytes_to_next_frame = frame_length
                .checked_sub(header_buffer.len() + xing_offset + xing_buffer.len())
                .ok_or(MP3DurationError {
                    kind: ErrorKind::MPEGFrameTooShort,
                    offset: bytes_read + xing_offset + xing_buffer.len(),
                    at_duration: duration,
                })?;

            if let Err(e) = skip(reader, &mut dump, bytes_to_next_frame) {
                bail!((bytes_read + xing_offset + xing_buffer.len(), duration), e);
            }
            bytes_read += frame_length - header_buffer.len();

            let frame_duration = (num_samples as u64 * 1_000_000_000) / (sampling_rate as u64);
            duration += Duration::new(0, frame_duration as u32);

            continue;
        }

        // ID3v2 frame
        let is_id3v2 = header_buffer[0] == 'I' as u8
            && header_buffer[1] == 'D' as u8
            && header_buffer[2] == '3' as u8;
        if is_id3v2 {
            let mut id3v2 = [0; 6]; // 4 bytes already read
            if let Err(e) = reader.read_exact(&mut id3v2) {
                bail!((bytes_read, duration), e);
            }
            let flags = id3v2[1];
            let footer_size: usize = if 0 != (flags & 0b0001_0000) { 10 } else { 0 };
            let tag_size: usize = ((id3v2[5] as u32)
                | ((id3v2[4] as u32) << 7)
                | ((id3v2[3] as u32) << 14)
                | ((id3v2[2] as u32) << 21)) as usize;
            if let Err(e) = skip(reader, &mut dump, tag_size + footer_size) {
                bail!((bytes_read + id3v2.len(), duration), e);
            }
            bytes_read += id3v2.len() + tag_size + footer_size;
            continue;
        }

        // ID3v1 frame
        let is_id3v1 = header_buffer[0] == 'T' as u8
            && header_buffer[1] == 'A' as u8
            && header_buffer[2] == 'G' as u8;
        if is_id3v1 {
            if let Err(e) = skip(reader, &mut dump, 128 - header_buffer.len()) {
                bail!((bytes_read, duration), e);
            }
            bytes_read += 128 - header_buffer.len();
            continue;
        }

        // APEv2 frame
        let maybe_is_ape_v2 = header_buffer[0] == 'A' as u8
            && header_buffer[1] == 'P' as u8
            && header_buffer[2] == 'E' as u8
            && header_buffer[3] == 'T' as u8;
        if maybe_is_ape_v2 {
            let mut ape_header = [0; 12];
            if let Err(e) = reader.read_exact(&mut ape_header[..]) {
                bail!((bytes_read, duration), e);
            }
            let is_really_ape_v2 = ape_header[0] == 'A' as u8
                && ape_header[1] == 'G' as u8
                && ape_header[2] == 'E' as u8
                && ape_header[3] == 'X' as u8;
            if !is_really_ape_v2 {
                bail!(
                    (bytes_read - header_buffer.len(), duration),
                    ErrorKind::UnexpectedFrame { header }
                );
            }
            let tag_size: usize = ((ape_header[8] as u32)
                | ((ape_header[9] as u32) << 8)
                | ((ape_header[10] as u32) << 16)
                | ((ape_header[11] as u32) << 24)) as usize;
            if let Err(e) = skip(reader, &mut dump, tag_size + 16) {
                bail!((bytes_read, duration), e);
            }
            bytes_read += ape_header.len() + tag_size + 16;
            continue;
        }

        bail!(
            (bytes_read - header_buffer.len(), duration),
            ErrorKind::UnexpectedFrame { header }
        );
    }

    Ok(duration)
}

/// Measures the duration of a file.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use std::fs::File;
/// use mp3_duration;
///
/// let path = Path::new("test/source.mp3");
/// let file = File::open(path).unwrap();
/// let duration = mp3_duration::from_file(&file).unwrap();
/// println!("File duration: {:?}", duration);
/// ```
pub fn from_file(file: &File) -> Result<Duration, MP3DurationError> {
    let mut reader = BufReader::new(file);
    from_read(&mut reader)
}

/// Measures the duration of a file.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use mp3_duration;
///
/// let path = Path::new("test/source.mp3");
/// let duration = mp3_duration::from_path(&path).unwrap();
/// println!("File duration: {:?}", duration);
/// ```
pub fn from_path<P>(path: P) -> Result<Duration, MP3DurationError>
where
    P: AsRef<Path>,
{
    File::open(path)
        .map_err(|e| MP3DurationError {
            kind: e.into(),
            offset: 0,
            at_duration: Duration::from_secs(0),
        })
        .and_then(|file| from_file(&file))
}

#[test]
fn lame_398_constant_bitrate_320() {
    let path = Path::new("test/CBR320.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(398, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(2 * 100_000_000 < nanos && nanos < 4 * 100_000_000);
}

#[test]
fn lame_398_variable_bitrate_v0() {
    let path = Path::new("test/VBR0.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(398, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(2 * 100_000_000 < nanos && nanos < 4 * 100_000_000);
}

#[test]
fn lame_398_variable_bitrate_v9() {
    let path = Path::new("test/VBR9.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(398, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(2 * 100_000_000 < nanos && nanos < 4 * 100_000_000);
}

#[test]
fn id3v1() {
    let path = Path::new("test/ID3v1.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(398, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(2 * 100_000_000 < nanos && nanos < 4 * 100_000_000);
}

#[test]
fn id3v2() {
    let path = Path::new("test/ID3v2.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(398, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(2 * 100_000_000 < nanos && nanos < 4 * 100_000_000);
}

#[test]
fn id3v2_with_image() {
    let path = Path::new("test/ID3v2WithImage.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(398, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(2 * 100_000_000 < nanos && nanos < 4 * 100_000_000);
}

#[test]
fn id3v2_empty() {
    let path = Path::new("test/SineEmptyID3.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(1, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(0 < nanos && nanos < 1 * 100_000_000);
}

#[test]
fn id3v2_bad_padding() {
    let path = Path::new("test/ID3v2WithBadPadding.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(398, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(2 < nanos && nanos < 3 * 100_000_000);
}

#[test]
fn apev2() {
    let path = Path::new("test/APEv2.mp3");
    let duration = from_path(path).unwrap();
    assert_eq!(398, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(2 < nanos && nanos < 4 * 100_000_000);
}

#[test]
fn bad_file() {
    let path = Path::new("test/piano.jpeg");
    let duration = from_path(path);
    assert!(duration.is_err());
}

#[test]
fn truncated() {
    let path = Path::new("test/Truncated.mp3");
    let error = from_path(path).unwrap_err();

    if let ErrorKind::UnexpectedEOF = error.kind {
        let duration = error.at_duration;
        assert_eq!(206, duration.as_secs());
        let nanos = duration.subsec_nanos();
        assert!(7 * 100_000_000 < nanos && nanos < 8 * 100_000_000);
    } else {
        panic!("error.kind must be ErrorKind::UnexpectedEOF")
    }
}

#[test]
fn mpeg_frame_too_short() {
    let path = Path::new("test/MPEGFrameTooShort.mp3");
    let error = from_path(path).unwrap_err();

    if let ErrorKind::MPEGFrameTooShort = error.kind {
        let duration = error.at_duration;
        assert_eq!(395, duration.as_secs());
        let nanos = duration.subsec_nanos();
        assert!(4 * 100_000_000 < nanos && nanos < 6 * 100_000_000);
    } else {
        panic!("error.kind must be ErrorKind::MPEGFrameTooShort")
    }
}
