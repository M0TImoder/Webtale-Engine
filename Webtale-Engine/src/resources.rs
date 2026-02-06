use bevy::prelude::*;
use pyo3::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ItemInfo {
    pub healAmount: i32,
    pub attack: i32,
    pub defense: i32,
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
    pub maxHp: f32,
    pub lv: i32,
    pub name: String,

    pub speed: f32,
    pub attack: f32,
    pub defense: f32,
    pub invincibilityDuration: f32,

    pub enemyHp: i32,
    pub enemyMaxHp: i32,
    pub enemyAtk: i32,
    pub enemyDef: i32,
    pub enemyName: String,
    pub enemyDialogText: String,
    pub enemyActCommands: Vec<String>,
    pub enemyActTexts: HashMap<String, String>,
    pub enemyBubbleMessages: Vec<String>,
    pub enemyBodyTexture: String,
    pub enemyHeadTexture: String,
    pub enemyHeadYOffset: f32,
    pub enemyBaseX: f32,
    pub enemyBaseY: f32,
    pub enemyScale: f32,
    
    pub enemyAttacks: Vec<String>,
    pub phaseName: String,
    pub phaseTurn: i32,
    pub turnCount: i32,
    pub enemyBubbleTexture: String,
    pub enemyBubbleMessageOverride: Option<String>,

    pub mnFight: i32,
    pub myFight: i32,

    pub menuLayer: i32, 
    pub menuCoords: Vec<i32>,

    pub inventory: Vec<String>,
    pub equippedItems: Vec<String>,
    pub itemPage: usize,

    pub dialogText: String,
    
    pub bubbleTimer: Timer,
    pub damageDisplayTimer: Timer,

    pub turnTimer: f32,
    
    pub invincibilityTimer: f32,
    pub lastPlayerAction: String,
    pub lastActCommand: Option<String>,
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
    pub hpLabel: Handle<Font>,
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
    pub currentTab: EditorTab,
}

#[derive(Resource, Default)]
pub struct EditorPreviewTexture(pub Handle<Image>);

#[derive(Resource, Default)]
pub struct DanmakuPreviewTexture(pub Handle<Image>);
