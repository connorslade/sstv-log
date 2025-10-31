#[derive(Debug)]
pub enum SstvMode {
    Martin1,
    Martin2,

    Scottie1,
    Scottie2,
    ScottieDX,

    Robot36,
    Robot72,

    Unknown(u8),
}

pub struct DecodeConfig {
    resolution: (u32, u32),
}

// const ABORT_TIMEOUT: f32 = 3.0;
// const MIN_ROW_DURATION: f32 = 0.2; // ← verify this

impl SstvMode {
    pub fn from_vis(vis: u8) -> Self {
        match vis {
            44 => SstvMode::Martin1,
            40 => SstvMode::Martin2,

            60 => SstvMode::Scottie1,
            56 => SstvMode::Scottie2,
            76 => SstvMode::ScottieDX,

            8 => SstvMode::Robot36,
            12 => SstvMode::Robot72,

            x => SstvMode::Unknown(x),
        }
    }

    pub fn config(&self) -> DecodeConfig {
        match self {
            SstvMode::Martin1 => DecodeConfig {
                resolution: (320, 256),
            },
            _ => unimplemented!(),
        }
    }
}
