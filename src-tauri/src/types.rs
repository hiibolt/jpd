use std::collections::HashMap;
use std::sync::{mpsc::{Sender, Receiver}, atomic::AtomicUsize};
use std::sync::{Arc, atomic::AtomicBool};

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum AppEvent {
    SwitchedWeapon {
        weapon_ind: usize,
    },
    StartedShooting {
        weapon_ind: usize
    },
    StoppedShooting
}
#[derive(Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub require_right_hold: bool,
}
#[derive(Clone)]
pub struct AppState {
    pub games:         Arc<RwLock<Vec<Game>>>,
    pub global_config: Arc<GlobalConfig>,
    
    pub events_channel_sender:   Arc<Sender<AppEvent>>,
    pub events_channel_reciever: Arc<Mutex<Receiver<AppEvent>>>,

    pub left_hold_active:  Arc<AtomicBool>,
    pub right_hold_active: Arc<AtomicBool>,
    pub current_game_index:     Arc<AtomicUsize>,
    pub current_category_index: Arc<AtomicUsize>,
    pub current_loadout_index:  Arc<AtomicUsize>,
    pub current_weapon_index:   Arc<AtomicUsize>,
}
#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize, Clone)]
pub struct FullAutoStandardConfig {
    pub name: String,
    pub description: Option<String>,
    pub rpm: u128,
    pub first_shot_scale: f32,
    pub exponential_factor: f32,
    pub dx: f32,
    pub dy: f32,
    pub mag_size: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    pub name:       String,
    pub categories: Vec<Category>,
    pub weapons:    HashMap<String, Weapon>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub loadouts: Vec<Loadout>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Loadout {
    pub name: String,
    pub icon_url: Option<String>,
    #[serde(default)]
    pub icon_only: bool,
    pub weapon_ids: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "config")]
pub enum Weapon {
    SingleFire(SingleFireConfig),
    FullAutoStandard(FullAutoStandardConfig),
}