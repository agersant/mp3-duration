use std::path::Path;

use crate::error::ErrorKind;
use crate::from_path;

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
    let duration = from_path(path).unwrap();
    assert_eq!(395, duration.as_secs());
    let nanos = duration.subsec_nanos();
    assert!(4 * 100_000_000 < nanos && nanos < 6 * 100_000_000);
}
