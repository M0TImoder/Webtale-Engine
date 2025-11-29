use bevy::prelude::*;

#[derive(Resource)]
pub struct GameState {
    pub hp: f32,
    pub max_hp: f32,
    pub lv: i32,
    pub name: String,
    
    pub enemy_hp: i32,
    pub enemy_max_hp: i32,
    pub enemy_def: i32,

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
