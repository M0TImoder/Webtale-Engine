use bevy::prelude::*;
use rand::Rng;
use rustpython_vm::function::ArgIntoFloat;
use rustpython_vm::Interpreter;
use rustpython_vm::PyObjectRef;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ItemInfo {
    pub heal_amount: i32,
    pub attack: i32,
    pub defense: i32,
    pub text: String,
}

#[derive(Resource, Default)]
pub struct ItemDictionary(pub HashMap<String, ItemInfo>);

#[derive(Resource, Default)]
pub struct DanmakuScripts {
    pub modules: HashMap<String, PyObjectRef>,
}

pub struct PythonRuntime {
    pub interpreter: Interpreter,
}

impl Default for PythonRuntime {
    fn default() -> Self {
        let interpreter = Interpreter::with_init(Default::default(), |vm| {
            vm.add_native_modules(rustpython_stdlib::get_module_inits());
            vm.add_frozen(rustpython_pylib::FROZEN_STDLIB);
        });
        interpreter.enter(|vm| {
            fn random_random() -> f64 {
                rand::thread_rng().gen::<f64>()
            }

            fn random_uniform(a: ArgIntoFloat, b: ArgIntoFloat) -> f64 {
                let a = f64::from(a);
                let b = f64::from(b);
                rand::thread_rng().gen_range(a..b)
            }

            let dict = vm.ctx.new_dict();
            let module = vm.new_module("random", dict.clone(), None);
            let random_fn = vm.new_function("random", random_random);
            let uniform_fn = vm.new_function("uniform", random_uniform);
            if let Err(err) = dict.set_item("random", random_fn.into(), vm) {
                vm.print_exception(err.clone());
            }
            if let Err(err) = dict.set_item("uniform", uniform_fn.into(), vm) {
                vm.print_exception(err.clone());
            }
            if let Ok(modules) = vm.sys_module.get_attr("modules", vm) {
                let _ = modules.set_item("random", module.into(), vm);
            }
        });
        Self { interpreter }
    }
}

// プレイヤーデータ
#[derive(Resource)]
pub struct PlayerState {
    pub hp: f32,
    pub max_hp: f32,
    pub lv: i32,
    pub name: String,
    pub speed: f32,
    pub attack: f32,
    pub defense: f32,
    pub invincibility_duration: f32,
    pub invincibility_timer: f32,
    pub inventory: Vec<String>,
    pub equipped_items: Vec<String>,
}

// 敵データ
#[derive(Resource)]
pub struct EnemyState {
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub name: String,
    pub dialog_text: String,
    pub act_commands: Vec<String>,
    pub act_texts: HashMap<String, String>,
    pub bubble_messages: Vec<String>,
    pub body_texture: String,
    pub head_texture: String,
    pub head_yoffset: f32,
    pub base_x: f32,
    pub base_y: f32,
    pub scale: f32,
    pub attacks: Vec<String>,
    pub bubble_texture: String,
    pub bubble_message_override: Option<String>,
    pub bubble_pos_override: Option<Vec2>,
}

// メニュー操作
#[derive(Resource)]
pub struct MenuState {
    pub menu_layer: i32,
    pub menu_coords: Vec<i32>,
    pub item_page: usize,
    pub dialog_text: String,
}

// 戦闘フロー制御
#[derive(Resource)]
pub struct CombatState {
    pub mn_fight: i32,
    pub my_fight: i32,
    pub phase_name: String,
    pub phase_turn: i32,
    pub turn_count: i32,
    pub turn_timer: f32,
    pub bubble_timer: Timer,
    pub damage_display_timer: Timer,
    pub last_player_action: String,
    pub last_act_command: Option<String>,
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
