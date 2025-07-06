use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{mpsc::{Sender, Receiver}, atomic::AtomicUsize};
use std::sync::{Arc, atomic::AtomicBool};

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

pub struct LoadedGames {
    pub game_data: Vec<Game>,
}
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum KeyStatus {
    Invalid { key: String },
    Valid { key: String, timestamp: u64 },
    Expired { key: String, timestamp: u64 },
    Banned { key: String }
}
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum KeyStatusResponse {
    Invalid { key: String },
    Valid { key: String, timestamp: u64, config: Game },
    Expired { key: String, timestamp: u64 },
    Banned { key: String }
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum AppEvent {
    UpdatedGames {
        games: Vec<Game>,
    },
    SwitchedWeapon {
        weapon_ind: usize,
    },
    StartedShooting {
        weapon_ind: usize
    },
    StoppedShooting
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeybindConfig {
    pub require_right_hold: bool,
    pub primary_weapon: char,
    pub secondary_weapon: char,
    pub alternative_fire: char,
}
impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            require_right_hold: true,
            primary_weapon: '1',
            secondary_weapon: '2',
            alternative_fire: '3',
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct MouseConfig {
    pub horizontal_multiplier: f32,
    pub vertical_multiplier: f32,
}
impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            horizontal_multiplier: 1.0,
            vertical_multiplier: 1.0,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub keybinds: KeybindConfig,
    #[serde(default)]
    pub mouse_config: MouseConfig,
}
impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            keybinds: KeybindConfig::default(),
            mouse_config: MouseConfig::default(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub games:           Arc<RwLock<Vec<Game>>>,
    pub global_config:   Arc<RwLock<GlobalConfig>>,
    pub assets_dir_path: Arc<PathBuf>,
    
    pub events_channel_sender:   Arc<Sender<AppEvent>>,
    pub events_channel_reciever: Arc<Mutex<Receiver<AppEvent>>>,

    pub left_hold_active:       Arc<AtomicBool>,
    pub right_hold_active:      Arc<AtomicBool>,
    pub current_game_index:     Arc<AtomicUsize>,
    pub current_category_index: Arc<AtomicUsize>,
    pub current_loadout_index:  Arc<AtomicUsize>,
    pub current_weapon_index:   Arc<AtomicUsize>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SingleFireConfig {
    pub name: String,
    pub description: Option<String>,
    pub trigger_delay_ms: u32,
    pub recoil_completion_ms: u32,
    pub release_delay_ms: u32,
    pub dx: f32,
    pub dy: f32,
    pub mag_size: u32,
    pub autofire: bool,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullAutoStandardConfig {
    pub name: String,
    pub description: Option<String>,
    pub rpm: u64,
    pub first_shot_scale: f32,
    pub exponential_factor: f32,
    pub dx: f32,
    pub dy: f32,
    pub mag_size: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Game {
    pub name:       String,
    pub key:        Option<String>,
    pub key_status: Option<KeyStatus>,
    pub categories: Option<Vec<Category>>,
    pub weapons:    Option<HashMap<String, Weapon>>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Category {
    pub name: String,
    pub loadouts: Vec<Loadout>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Loadout {
    pub name: String,
    pub icon_url: Option<String>,
    #[serde(default)]
    pub icon_only: bool,
    pub weapon_ids: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "config")]
pub enum Weapon {
    SingleFire(SingleFireConfig),
    FullAutoStandard(FullAutoStandardConfig),
}