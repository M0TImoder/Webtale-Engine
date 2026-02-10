use bevy::prelude::*;
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy_egui::EguiPlugin;

mod constants;
mod components;
mod resources;
mod python_scripts;
mod python_utils;
mod systems;

use constants::*;
use resources::*;
use systems::*;

fn game_running(state: Res<GameRunState>) -> bool {
    state.running
}

// アプリ起動
fn main() {
    App::new()
        // プラグイン設定
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        title: "Webtale Engine".to_string(),
                        resizable: true,
                        canvas: Some("#bevy".to_string()),
                        prevent_default_event_handling: false,
                        visible: false,
                        ..default()
                    }),
                    close_when_requested: false,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        // UIプラグイン
        .add_plugins(EguiPlugin)
        // クリアカラー
        .insert_resource(ClearColor(Color::BLACK))
        // バトルボックス初期値
        .insert_resource(BattleBox {
            current: Rect::new(32.0, 250.0, 602.0, 385.0),
            target: Rect::new(32.0, 250.0, 602.0, 385.0),
        })
        // エディタリソース
        .init_resource::<EditorState>()
        .init_resource::<EditorPreviewTexture>()
        .init_resource::<DanmakuPreviewTexture>()
        .init_resource::<DanmakuScripts>()
        .init_resource::<GameRunState>()
        // メニュー描画キャッシュ
        .init_resource::<MenuRenderCache>()
        // Python実行環境
        .insert_non_send_resource(PythonRuntime::default())
        // 起動システム
        .add_systems(Startup, (
            setup::setup,
            input::spawn_initial_editor_window,
        ))
        // 更新システム
        .add_systems(Update, input::handle_global_input)
        .add_systems(Update, setup::camera_scaling_system)
        .add_systems(Update, ui::draw_battle_box)
        .add_systems(Update, ui::draw_ui_status)
        .add_systems(Update, editor::editor_ui_system)
        .add_systems(Update, (
            input::menu_input_system,
            ui::menu_render_system,
            player::soul_position_sync,
            player::soul_combat_movement,
            ui::update_box_size,
            ui::update_button_sprites,
            ui::animate_text,
            ui::animate_enemy_head,
        ).run_if(game_running))
        // 戦闘システム
        .add_systems(Update, (
            combat::battle_flow_control,
            combat::attack_bar_update,
            combat::apply_pending_damage,   
            combat::animate_slice_effect,
            combat::damage_number_update,   
            combat::enemy_hp_bar_update,    
            combat::vaporize_enemy_system, 
            combat::dust_particle_update,
        ).run_if(game_running))
        .add_systems(Update, (
            combat::leapfrog_bullet_update,
            combat::combat_turn_manager,
            combat::soul_collision_detection,
            combat::invincibility_update,
            combat::heart_defeated_update,
            combat::heart_shard_update,
            combat::game_over_sequence_update,
        ).run_if(game_running))
        .run();
}
