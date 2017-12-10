[![Build Status](https://travis-ci.org/agersant/mp3-duration.svg?branch=master)](https://travis-ci.org/agersant/mp3-duration)

# mp3-duration

This crate has only one purpose: determining the playback duration of mp3 files.

## Example

```rust
use std::path::Path;
use std::fs::File;
use mp3_duration;

let path = Path::new("music.mp3");
let mut file = File::open(path).unwrap();
let duration = mp3_duration::from_file(&mut file).unwrap();
println!("File duration: {:?}", duration);
```

## Links
- [Format reference](https://www.codeproject.com/Articles/8295/MPEG-Audio-Frame-Header)
- [Audio test file source](http://freemusicarchive.org/music/Karine_Gilanyan/Beethovens_Sonata_No_15_in_D_Major/Beethoven_-_Piano_Sonata_nr15_in_D_major_op28_Pastoral_-_I_Allegro)
- [Non-audio test file source](https://www.pexels.com/photo/close-up-of-tiled-floor-258805/)
