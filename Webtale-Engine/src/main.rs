use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy_egui::EguiPlugin;

mod constants;
mod components;
mod resources;
mod systems;

use constants::*;
use resources::*;
use systems::*;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        title: "Webtale Engine".to_string(),
                        resizable: true,
                        canvas: Some("#bevy".to_string()),
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(EguiPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(BattleBox {
            current: Rect::new(32.0, 250.0, 602.0, 385.0),
            target: Rect::new(32.0, 250.0, 602.0, 385.0),
        })
        .init_resource::<EditorState>()
        .init_resource::<EditorPreviewTexture>()
        .init_resource::<DanmakuPreviewTexture>()
        .init_resource::<DanmakuScripts>()
        .add_systems(Startup, setup::setup)
        .add_systems(Update, (
            input::handle_global_input,
            setup::camera_scaling_system,
            input::menu_input_system,
            ui::menu_render_system,
            player::soul_position_sync,
            player::soul_combat_movement,
            ui::update_box_size,
            ui::draw_battle_box,
            ui::draw_ui_status,
            ui::update_button_sprites,
            ui::animate_text,
            ui::animate_enemy_head, 
            editor::editor_ui_system,
        ))
        .add_systems(Update, (
            combat::battle_flow_control,
            combat::attack_bar_update,
            combat::apply_pending_damage,   
            combat::animate_slice_effect,
            combat::damage_number_update,   
            combat::enemy_hp_bar_update,    
            combat::vaporize_enemy_system, 
            combat::dust_particle_update,
            combat::leapfrog_bullet_update,
            combat::combat_turn_manager,
            combat::soul_collision_detection,
            combat::invincibility_update,
            combat::heart_defeated_update,
            combat::heart_shard_update,
            combat::game_over_sequence_update,
        ))
        .run();
}
