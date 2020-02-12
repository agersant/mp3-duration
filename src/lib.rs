extern crate failure;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

mod constants;
mod context;
mod error;
#[cfg(test)]
mod test;

use crate::constants::*;
use crate::context::Context;
use crate::error::*;

pub use crate::error::MP3DurationError;

fn get_bitrate<T: Read>(
    context: &Context<T>,
    version: Version,
    layer: Layer,
    encoded_bitrate: u8,
) -> Result<u32, MP3DurationError> {
    if encoded_bitrate >= 15 {
        return Err(context.error(ErrorKind::InvalidBitrate {
            bitrate: encoded_bitrate,
        }));
    }
    if layer == Layer::NotDefined {
        return Err(context.error(ErrorKind::ForbiddenLayer));
    }
    Ok(1000 * BIT_RATES[version as usize][layer as usize][encoded_bitrate as usize])
}

fn get_sampling_rate<T: Read>(
    context: &Context<T>,
    version: Version,
    encoded_sampling_rate: u8,
) -> Result<u32, MP3DurationError> {
    if encoded_sampling_rate >= 3 {
        return Err(context.error(ErrorKind::InvalidSamplingRate {
            sampling_rate: encoded_sampling_rate,
        }));
    }
    Ok(SAMPLING_RATES[version as usize][encoded_sampling_rate as usize])
}

fn get_samples_per_frame<T: Read>(
    context: &Context<T>,
    version: Version,
    layer: Layer,
) -> Result<u32, MP3DurationError> {
    if layer == Layer::NotDefined {
        return Err(context.error(ErrorKind::ForbiddenLayer));
    }
    Ok(SAMPLES_PER_FRAME[version as usize][layer as usize])
}

fn get_side_information_size(version: Version, mode: Mode) -> usize {
    SIDE_INFORMATION_SIZES[version as usize][mode as usize] as usize
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

    let mut context = Context::new(reader);

    loop {
        // Skip over all 0x00 bytes (these are probably incorrectly added padding bytes for id3v2)
        header_buffer[0] = 0;
        while header_buffer[0] == 0 {
            match context.read_exact(&mut header_buffer[0..1]) {
                Ok(_) => (),
                Err(_) if context.reached_eof() => break,
                Err(e) => return Err(e),
            };
        }

        match context.read_exact(&mut header_buffer[1..]) {
            Ok(_) => (),
            Err(_) if context.reached_eof() => break,
            Err(e) => return Err(e),
        };

        // MPEG frame
        let header = (header_buffer[0] as u32) << 24
            | (header_buffer[1] as u32) << 16
            | (header_buffer[2] as u32) << 8
            | header_buffer[3] as u32;
        let is_mp3 = header >> 21 == 0x7FF;
        if is_mp3 {
            let version = match (header >> 19) & 0b11 {
                0 => Version::Mpeg25,
                1 => return Err(context.error(ErrorKind::ForbiddenVersion)),
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

            let sampling_rate = get_sampling_rate(&context, version, encoded_sampling_rate as u8)?;
            let num_samples = get_samples_per_frame(&context, version, layer)?;

            let xing_offset = get_side_information_size(version, mode);
            let mut xing_buffer = [0; 12];

            context.skip(xing_offset)?;
            context.read_exact(&mut xing_buffer)?;

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

            let bitrate = get_bitrate(&context, version, layer, encoded_bitrate as u8)?;
            let frame_length = (num_samples / 8 * bitrate / sampling_rate + padding) as usize;

            let bytes_to_next_frame = frame_length
                .checked_sub(header_buffer.len() + xing_offset + xing_buffer.len())
                .ok_or(context.error(ErrorKind::MPEGFrameTooShort))?;

            context.skip(bytes_to_next_frame)?;

            let frame_duration = (num_samples as u64 * 1_000_000_000) / (sampling_rate as u64);
            context.duration += Duration::new(0, frame_duration as u32);

            continue;
        }

        // ID3v2 frame
        let is_id3v2 = header_buffer[0] == 'I' as u8
            && header_buffer[1] == 'D' as u8
            && header_buffer[2] == '3' as u8;
        if is_id3v2 {
            let mut id3v2 = [0; 6]; // 4 bytes already read
            context.read_exact(&mut id3v2)?;
            let flags = id3v2[1];
            let footer_size: usize = if 0 != (flags & 0b0001_0000) { 10 } else { 0 };
            let tag_size: usize = ((id3v2[5] as u32)
                | ((id3v2[4] as u32) << 7)
                | ((id3v2[3] as u32) << 14)
                | ((id3v2[2] as u32) << 21)) as usize;
            context.skip(tag_size + footer_size)?;
            continue;
        }

        // ID3v1 frame
        let is_id3v1 = header_buffer[0] == 'T' as u8
            && header_buffer[1] == 'A' as u8
            && header_buffer[2] == 'G' as u8;
        if is_id3v1 {
            context.skip(128 - header_buffer.len())?;
            continue;
        }

        // APEv2 frame
        let maybe_is_ape_v2 = header_buffer[0] == 'A' as u8
            && header_buffer[1] == 'P' as u8
            && header_buffer[2] == 'E' as u8
            && header_buffer[3] == 'T' as u8;
        if maybe_is_ape_v2 {
            let mut ape_header = [0; 12];
            context.read_exact(&mut ape_header)?;
            let is_really_ape_v2 = ape_header[0] == 'A' as u8
                && ape_header[1] == 'G' as u8
                && ape_header[2] == 'E' as u8
                && ape_header[3] == 'X' as u8;
            if is_really_ape_v2 {
                let tag_size: usize = ((ape_header[8] as u32)
                    | ((ape_header[9] as u32) << 8)
                    | ((ape_header[10] as u32) << 16)
                    | ((ape_header[11] as u32) << 24))
                    as usize;
                context.skip(tag_size + 16)?;
                continue;
            }
        }

        return Err(context.error(ErrorKind::UnexpectedFrame { header }));
    }

    Ok(context.duration)
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
