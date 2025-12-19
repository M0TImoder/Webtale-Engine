use bevy::prelude::*;
use pyo3::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ItemInfo {
    pub heal_amount: i32,
    pub text: String,
}

#[derive(Resource, Default)]
pub struct ItemDictionary(pub HashMap<String, ItemInfo>);

#[derive(Resource, Default)]
pub struct DanmakuScripts {
    pub modules: HashMap<String, PyObject>,
}

#[derive(Resource)]
pub struct GameState {
    pub hp: f32,
    pub max_hp: f32,
    pub lv: i32,
    pub name: String,

    pub speed: f32,
    pub attack: f32,
    pub invincibility_duration: f32,
    
    pub enemy_hp: i32,
    pub enemy_max_hp: i32,
    pub enemy_def: i32,
    
    pub enemy_attacks: Vec<String>,

    pub mnfight: i32,
    pub myfight: i32,

    pub menu_layer: i32, 
    pub menu_coords: Vec<i32>,

    pub inventory: Vec<String>,
    pub item_page: usize,

    pub dialog_text: String,
    
    pub bubble_timer: Timer,
    pub damage_display_timer: Timer,

    pub turntimer: f32,
    
    pub invincibility_timer: f32,
}

#[derive(Resource)]
pub struct BattleBox {
    pub current: Rect,
    pub target: Rect,
}

#[derive(Resource)]
pub struct GameFonts {
    pub main: Handle<Font>,
    pub dialog: Handle<Font>,
    pub hp_label: Handle<Font>,
    pub damage: Handle<Font>, 
}

#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum EditorTab {
    #[default]
    Battle,
    DanmakuPreview,
}

#[derive(Resource, Default)]
pub struct EditorState {
    pub current_tab: EditorTab,
}

#[derive(Resource, Default)]
pub struct EditorPreviewTexture(pub Handle<Image>);

#[derive(Resource, Default)]
pub struct DanmakuPreviewTexture(pub Handle<Image>);
