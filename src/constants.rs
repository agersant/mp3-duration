#[derive(Clone, Copy, Debug)]
pub enum Version {
    Mpeg1,
    Mpeg2,
    Mpeg25,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Layer {
    NotDefined,
    Layer1,
    Layer2,
    Layer3,
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Stereo,
    JointStereo,
    DualChannel,
    Mono,
}

pub static BIT_RATES: [[[u32; 16]; 4]; 3] = [
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

pub static SAMPLING_RATES: [[u32; 4]; 3] = [
    [44100, 48000, 32000, 0], // Mpeg1
    [22050, 24000, 16000, 0], // Mpeg2
    [11025, 12000, 8000, 0],  // Mpeg25
];

pub static SAMPLES_PER_FRAME: [[u32; 4]; 3] = [
    [0, 384, 1152, 1152], // Mpeg1
    [0, 384, 1152, 576],  // Mpeg2
    [0, 384, 1152, 576],  // Mpeg25
];

pub static SIDE_INFORMATION_SIZES: [[u32; 4]; 3] = [
    [32, 32, 32, 17], // Mpeg1
    [17, 17, 17, 9],  // Mpeg2
    [17, 17, 17, 9],  // Mpeg25
];
