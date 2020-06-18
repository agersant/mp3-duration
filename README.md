[![Crates.io](https://img.shields.io/crates/v/mp3-duration.svg)](https://crates.io/crates/mp3-duration)
[![Build Status](https://travis-ci.org/agersant/mp3-duration.svg?branch=master)](https://travis-ci.org/agersant/mp3-duration)

# mp3-duration

This crate has only one purpose: determining the playback duration of mp3 files.

## Example

```rust
use std::path::Path;
use mp3_duration;

let path = Path::new("music.mp3");
let duration = mp3_duration::from_path(&path).unwrap();
println!("File duration: {:?}", duration);
```

##  Changelog

### Version 0.1.10
- Replaced usage of `failure` with `thiserror` for error management (thanks @amesgen for the contribution)

### Version 0.1.9
- Fixed a bug where the MP3Duration error type was no longer public since version 0.1.8 (thanks @compenguy for the contribution)

### Version 0.1.8
- Minor performance improvements

### Version 0.1.7
- Fixed a crash when reading corrupted files with impossibly short MPEG frames

## Links
- [Format reference](https://www.codeproject.com/Articles/8295/MPEG-Audio-Frame-Header)
- [Audio test file source](http://freemusicarchive.org/music/Karine_Gilanyan/Beethovens_Sonata_No_15_in_D_Major/Beethoven_-_Piano_Sonata_nr15_in_D_major_op28_Pastoral_-_I_Allegro)
- [Non-audio test file source](https://www.pexels.com/photo/close-up-of-tiled-floor-258805/)
