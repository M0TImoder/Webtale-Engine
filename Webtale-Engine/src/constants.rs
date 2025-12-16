use bevy::prelude::Color;
use bevy::prelude::*;

pub const PROJECT_NAME: &str = "default";

pub const WINDOW_WIDTH: f32 = 640.0;
pub const WINDOW_HEIGHT: f32 = 480.0;
pub const ORIGIN_X: f32 = -320.0;
pub const ORIGIN_Y: f32 = 240.0;

pub const COLOR_HP_RED: Color = Color::rgb(1.0, 0.0, 0.0);
pub const COLOR_HP_YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
pub const COLOR_UI_TEXT: Color = Color::WHITE;

pub const BUTTON_Y_GML: f32 = 432.0;
pub const BTN_FIGHT_X: f32 = 32.0;
pub const BTN_ACT_X: f32 = 185.0;
pub const BTN_ITEM_X: f32 = 345.0;
pub const BTN_MERCY_X: f32 = 500.0;

pub const Z_ENEMY_BODY: f32 = 3.0;
pub const Z_ENEMY_HEAD: f32 = 4.0;

pub const Z_BORDER: f32 = 5.0;
pub const Z_BG: f32 = 6.0;

pub const Z_ATTACK_TARGET: f32 = 10.0;
pub const Z_ATTACK_BAR: f32 = 11.0;

pub const Z_SLICE: f32 = 15.0;

pub const Z_BUTTON: f32 = 20.0;
pub const Z_HP_BAR_BG: f32 = 20.0;
pub const Z_HP_BAR_FG: f32 = 21.0;
pub const Z_TEXT: f32 = 22.0;

pub const Z_DAMAGE_HP_BAR: f32 = 25.0; 
pub const Z_DAMAGE_TEXT: f32 = 26.0;   

pub const Z_BUBBLE: f32 = 30.0; 
pub const Z_BUBBLE_TEXT: f32 = 31.0;

pub const Z_SOUL: f32 = 40.0;

pub const Z_GAMEOVER_BG: f32 = 100.0;
pub const Z_GAMEOVER_SOUL: f32 = 110.0;

pub const MENU_LAYER_TOP: i32 = 0;
pub const MENU_LAYER_FIGHT_TARGET: i32 = 1;
pub const MENU_LAYER_ACT_TARGET: i32 = 2;
pub const MENU_LAYER_ITEM: i32 = 3;
pub const MENU_LAYER_MERCY: i32 = 4;
pub const MENU_LAYER_ACT_COMMAND: i32 = 10;

pub const ITEMS_PER_PAGE: usize = 4;

pub fn gml_to_bevy(x: f32, y: f32) -> Vec3 {
    Vec3::new(ORIGIN_X + x, ORIGIN_Y - y, 0.0)
}
