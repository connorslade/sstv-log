use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

    pub fn to_vis(self) -> u8 {
        match self {
            SstvMode::Martin1 => 44,
            SstvMode::Martin2 => 40,

            SstvMode::Scottie1 => 60,
            SstvMode::Scottie2 => 56,
            SstvMode::ScottieDX => 76,

            SstvMode::Robot36 => 8,
            SstvMode::Robot72 => 12,

            SstvMode::Unknown(_) => 0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SstvMode::Martin1 => "Martin 1",
            SstvMode::Martin2 => "Martin 2",
            SstvMode::Scottie1 => "Scottie 1",
            SstvMode::Scottie2 => "Scottie 2",
            SstvMode::ScottieDX => "Scottie DX",
            SstvMode::Robot36 => "Robot 36",
            SstvMode::Robot72 => "Robot 72",
            SstvMode::Unknown(_) => "Unknown",
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
