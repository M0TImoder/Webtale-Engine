use bevy::prelude::*;
use rand::Rng;
use rustpython_vm::function::ArgIntoFloat;
use rustpython_vm::Interpreter;
use rustpython_vm::PyObjectRef;
use evalexpr::Node;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// アイテム情報
#[derive(Clone, Debug)]
pub struct ItemInfo {
    pub heal_amount: i32,
    pub attack: i32,
    pub defense: i32,
    pub text: String,
}

// アイテム辞書
#[derive(Resource, Default)]
pub struct ItemDictionary(pub HashMap<String, ItemInfo>);

// 弾幕式
#[derive(Clone)]
pub struct ExprAssignment {
    pub target: String,
    pub expr: Node,
}

#[derive(Clone)]
pub struct RustSimSpec {
    pub update_exprs: Vec<ExprAssignment>,
    pub delete_expr: Option<Node>,
    pub texture_expr: Option<Node>,
}

// 弾幕スクリプトキャッシュ
#[derive(Resource, Default)]
pub struct DanmakuScripts {
    pub modules: HashMap<String, PyObjectRef>,
    pub rust_specs: HashMap<String, RustSimSpec>,
}

// Python実行環境
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
    pub tachie_script: String,
    pub head_sway_speed: f32,
    pub head_sway_amplitude: f32,
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

// メニュー描画キー
#[derive(Clone, PartialEq)]
pub struct MenuRenderKey {
    pub menu_layer: i32,
    pub menu_coords: Vec<i32>,
    pub item_page: usize,
    pub dialog_text: String,
    pub enemy_name: String,
    pub enemy_hp: i32,
    pub enemy_max_hp: i32,
    pub act_commands: Vec<String>,
    pub inventory: Vec<String>,
}

// メニュー描画キャッシュ
#[derive(Resource, Default)]
pub struct MenuRenderCache {
    pub key: Option<MenuRenderKey>,
}

// 戦闘メイン状態
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainFightState {
    Menu,
    EnemyDialog,
    EnemyAttack,
    TurnCleanup,
    PlayerAttackBar,
    PlayerAttackResolve,
    PlayerDefeated,
}

impl Default for MainFightState {
    fn default() -> Self {
        Self::Menu
    }
}

// メッセージ状態
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageFightState {
    None,
    PlayerActionText,
}

impl Default for MessageFightState {
    fn default() -> Self {
        Self::None
    }
}

// 戦闘フロー制御
#[derive(Resource)]
pub struct CombatState {
    pub mn_fight: MainFightState,
    pub my_fight: MessageFightState,
    pub phase_name: String,
    pub phase_turn: i32,
    pub turn_count: i32,
    pub turn_timer: f32,
    pub bubble_timer: Timer,
    pub damage_display_timer: Timer,
    pub last_player_action: String,
    pub last_act_command: Option<String>,
}

// バトルボックス
#[derive(Resource)]
pub struct BattleBox {
    pub current: Rect,
    pub target: Rect,
}

// フォント管理
#[derive(Resource)]
pub struct GameFonts {
    pub main: Handle<Font>,
    pub dialog: Handle<Font>,
    pub hp_label: Handle<Font>,
    pub damage: Handle<Font>, 
}

// エディタタブ
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum EditorTab {
    #[default]
    Battle,
    DanmakuPreview,
    Settings,
    LeftPane,
    BottomPane,
}

// エディタ状態
#[derive(Resource, Default)]
pub struct EditorState {
    pub current_tab: EditorTab,
    pub font_configured: bool,
    pub preview_active: bool,
    pub controls_pinned: bool,
}

// プロジェクトブラウザ状態
#[derive(Resource)]
pub struct ProjectBrowserState {
    pub current_project: Option<PathBuf>,
    pub selected_folder: Option<PathBuf>,
    pub selected_file: Option<PathBuf>,
    pub renaming_file: Option<PathBuf>,
    pub rename_buffer: String,
    pub rename_extension: String,
    pub rename_focus_requested: bool,
    pub recent_projects: Vec<PathBuf>,
    pub expanded_folders: HashSet<PathBuf>,
    pub tree_width: f32,
}

impl Default for ProjectBrowserState {
    fn default() -> Self {
        Self {
            current_project: None,
            selected_folder: None,
            selected_file: None,
            renaming_file: None,
            rename_buffer: String::new(),
            rename_extension: String::new(),
            rename_focus_requested: false,
            recent_projects: Vec::new(),
            expanded_folders: HashSet::new(),
            tree_width: 200.0,
        }
    }
}

// ゲーム実行状態
#[derive(Resource, Default)]
pub struct GameRunState {
    pub running: bool,
    pub reset_requested: bool,
}

// エディタプレビュー
#[derive(Resource, Default)]
pub struct EditorPreviewTexture(pub Handle<Image>);

// 弾幕プレビュー
#[derive(Resource, Default)]
pub struct DanmakuPreviewTexture(pub Handle<Image>);
