use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Target emulation platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Nintendo64,
    PlayStation1,
    PlayStation2,
    SuperNintendo,
    NintendoEntertainmentSystem,
    SegaGenesis,
}

impl Platform {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Nintendo64 => "Nintendo 64",
            Self::PlayStation1 => "Sony PlayStation",
            Self::PlayStation2 => "Sony PlayStation 2",
            Self::SuperNintendo => "Super Nintendo Entertainment System",
            Self::NintendoEntertainmentSystem => "Nintendo Entertainment System",
            Self::SegaGenesis => "Sega Genesis / Mega Drive",
        }
    }

    pub fn default_core_name(&self) -> &'static str {
        match self {
            Self::Nintendo64 => {
                if std::path::Path::new("cores/parallel_n64_libretro.so").exists() {
                    "parallel_n64"
                } else {
                    "mupen64plus_next"
                }
            }
            Self::PlayStation1 => {
                if std::path::Path::new("cores/swanstation_libretro.so").exists() {
                    "swanstation"
                } else {
                    "pcsx_rearmed"
                }
            }
            Self::PlayStation2 => "play",
            Self::SuperNintendo => "snes9x",
            Self::NintendoEntertainmentSystem => "nestopia",
            Self::SegaGenesis => "genesis_plus_gx",
        }
    }

    pub fn matches_extension(&self, ext: &str) -> bool {
        let ext = ext.to_ascii_lowercase();
        match self {
            Self::Nintendo64 => matches!(ext.as_str(), "z64" | "n64" | "v64"),
            Self::PlayStation1 => matches!(ext.as_str(), "cue" | "iso" | "chd" | "bin" | "pbp"),
            Self::PlayStation2 => matches!(ext.as_str(), "iso" | "chd" | "cso" | "bin" | "elf"),
            Self::SuperNintendo => matches!(ext.as_str(), "sfc" | "smc"),
            Self::NintendoEntertainmentSystem => matches!(ext.as_str(), "nes"),
            Self::SegaGenesis => matches!(ext.as_str(), "md" | "bin" | "gen"),
        }
    }
}

/// Physical media format that can be inserted into consoles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalMedia {
    Cartridge(CartridgeMedia),
    OpticalDisc(DiscMedia),
}

impl PhysicalMedia {
    pub fn platform(&self) -> Platform {
        match self {
            Self::Cartridge(c) => c.platform,
            Self::OpticalDisc(d) => d.platform,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Cartridge(c) => &c.title,
            Self::OpticalDisc(d) => &d.title,
        }
    }

    pub fn rom_path(&self) -> &Path {
        match self {
            Self::Cartridge(c) => &c.rom_path,
            Self::OpticalDisc(d) => &d.disc_path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartridgeMedia {
    pub id: String,
    pub title: String,
    pub platform: Platform,
    pub rom_path: PathBuf,
    pub label_texture_path: Option<PathBuf>,
    pub plastic_color_rgba: [u8; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscMedia {
    pub id: String,
    pub title: String,
    pub platform: Platform,
    pub disc_path: PathBuf,
    pub label_texture_path: Option<PathBuf>,
    pub is_dvd: bool,
}
