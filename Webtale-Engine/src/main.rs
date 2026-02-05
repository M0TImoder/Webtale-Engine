#![allow(non_snake_case)]

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
                        visible: false,
                        ..default()
                    }),
                    close_when_requested: false,
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
        .add_systems(Startup, (
            setup::setup,
            input::spawnInitialEditorWindow,
        ))
        .add_systems(Update, (
            input::handleGlobalInput,
            setup::cameraScalingSystem,
            input::menuInputSystem,
            ui::menuRenderSystem,
            player::soulPositionSync,
            player::soulCombatMovement,
            ui::updateBoxSize,
            ui::drawBattleBox,
            ui::drawUiStatus,
            ui::updateButtonSprites,
            ui::animateText,
            ui::animateEnemyHead, 
            editor::editorUiSystem,
        ))
        .add_systems(Update, (
            combat::battleFlowControl,
            combat::attackBarUpdate,
            combat::applyPendingDamage,   
            combat::animateSliceEffect,
            combat::damageNumberUpdate,   
            combat::enemyHpBarUpdate,    
            combat::vaporizeEnemySystem, 
            combat::dustParticleUpdate,
            combat::leapfrogBulletUpdate,
            combat::combatTurnManager,
            combat::soulCollisionDetection,
            combat::invincibilityUpdate,
            combat::heartDefeatedUpdate,
            combat::heartShardUpdate,
            combat::gameOverSequenceUpdate,
        ))
        .run();
}
