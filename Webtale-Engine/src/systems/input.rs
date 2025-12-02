use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::app::AppExit;
use bevy::sprite::Anchor;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;
use crate::systems::setup::spawn_game_objects; 
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;

pub fn handle_global_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    mut exit_writer: EventWriter<AppExit>,
    asset_server: Res<AssetServer>,
    game_fonts: Res<GameFonts>,
    cleanup_query: Query<Entity, With<Cleanup>>,
    editor_window_query: Query<Entity, With<EditorWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        exit_writer.send(AppExit::default());
    }

    if (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) && input.just_pressed(KeyCode::Enter) {
        if let Ok(mut window) = window_query.get_single_mut() {
             window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen,
                _ => WindowMode::Windowed,
            };
        }
    }

    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) && input.just_pressed(KeyCode::KeyR) {
        for entity in cleanup_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        commands.insert_resource(BattleBox {
            current: Rect::new(32.0, 250.0, 602.0, 385.0),
            target: Rect::new(32.0, 250.0, 602.0, 385.0),
        });

        spawn_game_objects(&mut commands, &asset_server, &game_fonts);
    }

    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) && input.just_pressed(KeyCode::KeyE) {
        if editor_window_query.is_empty() {
            let editor_window = commands.spawn((
                Window {
                    title: "Danmaku Editor".to_string(),
                    resolution: (1280.0, 720.0).into(),
                    resizable: true,
                    prevent_default_event_handling: false, 
                    ..default()
                },
                EditorWindow,
            )).id();

            let size = Extent3d {
                width: 640,
                height: 480,
                ..default()
            };
            let mut image = Image {
                texture_descriptor: bevy::render::render_resource::TextureDescriptor {
                    label: Some("Preview Texture"),
                    size,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Bgra8UnormSrgb,
                    mip_level_count: 1,
                    sample_count: 1,
                    usage: TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
                ..default()
            };
            image.resize(size);
            let image_handle = images.add(image);

            commands.spawn((
                Camera2dBundle {
                    camera: Camera {
                        target: RenderTarget::Image(image_handle.clone()),
                        order: -1,
                        ..default()
                    },
                    projection: OrthographicProjection {
                        scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(480.0),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 999.9), 
                    ..default()
                },
                RenderLayers::layer(0),
                EditorWindow, 
            ));

            commands.spawn((
                Camera2dBundle {
                    camera: Camera {
                        target: RenderTarget::Window(WindowRef::Entity(editor_window)),
                        clear_color: ClearColorConfig::Custom(Color::hex("222222").unwrap()), 
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 999.9),
                    ..default()
                },
                RenderLayers::layer(1),
                EditorWindow,
            ));

            commands.spawn((
                SpriteBundle {
                    texture: image_handle,
                    transform: Transform::from_xyz(0.0, 110.0, 0.0), 
                    ..default()
                },
                RenderLayers::layer(1),
                EditorWindow,
            ));
        }
    }
}

pub fn menu_input_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut typewriter_query: Query<(Entity, &mut Typewriter), With<MainDialogText>>,
    act_commands_query: Query<&ActCommands, With<EnemyBody>>,
    menu_items_query: Query<Entity, With<MenuTextItem>>, 
) {
    if game_state.mnfight != 0 || game_state.myfight != 0 { return; }
    let layer = game_state.menu_layer;
    let cursor_idx = game_state.menu_coords[layer as usize] as usize;
    
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        if layer == MENU_LAYER_TOP {
            game_state.menu_coords[layer as usize] = (game_state.menu_coords[layer as usize] - 1 + 4) % 4;
        } else if layer == MENU_LAYER_ACT_COMMAND {
             if cursor_idx % 2 == 1 { game_state.menu_coords[layer as usize] -= 1; }
        } else if layer == MENU_LAYER_ITEM {
            if cursor_idx % 2 == 1 { 
                game_state.menu_coords[layer as usize] -= 1; 
            } else if game_state.item_page > 0 {
                game_state.item_page -= 1;
                game_state.menu_coords[layer as usize] += 1; 
            }
        }
    }
    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        if layer == MENU_LAYER_TOP {
            game_state.menu_coords[layer as usize] = (game_state.menu_coords[layer as usize] + 1) % 4;
        } else if layer == MENU_LAYER_ACT_COMMAND {
             if cursor_idx % 2 == 0 {
                 if let Some(acts) = act_commands_query.iter().next() {
                     if cursor_idx + 1 < acts.commands.len() { game_state.menu_coords[layer as usize] += 1; }
                 }
             }
        } else if layer == MENU_LAYER_ITEM {
            let items_on_page = game_state.inventory.len().saturating_sub(game_state.item_page * ITEMS_PER_PAGE).min(ITEMS_PER_PAGE);
            if cursor_idx % 2 == 0 && cursor_idx + 1 < items_on_page {
                game_state.menu_coords[layer as usize] += 1;
            } else if cursor_idx % 2 == 1 && (game_state.item_page + 1) * ITEMS_PER_PAGE < game_state.inventory.len() {
                game_state.item_page += 1;
                game_state.menu_coords[layer as usize] -= 1; 
            }
        }
    }
    if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::KeyW) {
         if layer == MENU_LAYER_ACT_COMMAND && cursor_idx >= 2 { game_state.menu_coords[layer as usize] -= 2; }
         else if layer == MENU_LAYER_ITEM && cursor_idx >= 2 { game_state.menu_coords[layer as usize] -= 2; }
         else if layer == MENU_LAYER_MERCY && cursor_idx > 0 { game_state.menu_coords[layer as usize] -= 1; }
    }
    if input.just_pressed(KeyCode::ArrowDown) || input.just_pressed(KeyCode::KeyS) {
         if layer == MENU_LAYER_ACT_COMMAND {
            if let Some(acts) = act_commands_query.iter().next() {
                 if cursor_idx + 2 < acts.commands.len() { game_state.menu_coords[layer as usize] += 2; }
            }
         } else if layer == MENU_LAYER_ITEM {
            let items_on_page = game_state.inventory.len().saturating_sub(game_state.item_page * ITEMS_PER_PAGE).min(ITEMS_PER_PAGE);
            if cursor_idx + 2 < items_on_page { game_state.menu_coords[layer as usize] += 2; }
         } else if layer == MENU_LAYER_MERCY {
             if cursor_idx < 1 { game_state.menu_coords[layer as usize] += 1; }
         }
    }

    if input.just_pressed(KeyCode::KeyZ) {
        match layer {
            MENU_LAYER_TOP => {
                let selected = game_state.menu_coords[MENU_LAYER_TOP as usize];
                match selected {
                    0 => { game_state.menu_layer = MENU_LAYER_FIGHT_TARGET; game_state.menu_coords[MENU_LAYER_FIGHT_TARGET as usize] = 0; },
                    1 => { game_state.menu_layer = MENU_LAYER_ACT_TARGET; game_state.menu_coords[MENU_LAYER_ACT_TARGET as usize] = 0; },
                    2 => { 
                        if !game_state.inventory.is_empty() {
                            game_state.menu_layer = MENU_LAYER_ITEM; 
                            game_state.menu_coords[MENU_LAYER_ITEM as usize] = 0; 
                            game_state.item_page = 0;
                        }
                    }, 
                    3 => { game_state.menu_layer = MENU_LAYER_MERCY; game_state.menu_coords[MENU_LAYER_MERCY as usize] = 0; }, 
                    _ => {}
                }
            },
            MENU_LAYER_FIGHT_TARGET => {
                game_state.mnfight = 4; 
                let box_center = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 250.0 + (385.0-250.0)/2.0);
                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("spr_target.png"),
                        sprite: Sprite { custom_size: Some(Vec2::new(566.0, 120.0)), ..default() },
                        transform: Transform::from_translation(box_center + Vec3::new(0.0, 0.0, Z_ATTACK_TARGET)),
                        ..default()
                    },
                    AttackTargetBox,
                    Cleanup,
                ));
                let bar_start_x = gml_to_bevy(32.0, 0.0).x;
                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("spr_targetchoice_1.png"),
                        sprite: Sprite { custom_size: Some(Vec2::new(14.0, 120.0)), ..default() },
                        transform: Transform::from_translation(Vec3::new(bar_start_x, box_center.y, Z_ATTACK_BAR)),
                        ..default()
                    },
                    AttackBar { speed: 420.0, moving: true, flash_timer: Timer::from_seconds(0.08, TimerMode::Repeating), flash_state: true },
                    Cleanup,
                ));

                for entity in menu_items_query.iter() { commands.entity(entity).despawn(); }
                if let Ok((entity, _)) = typewriter_query.get_single_mut() { commands.entity(entity).despawn(); }
            },
            MENU_LAYER_ACT_TARGET => {
                game_state.menu_layer = MENU_LAYER_ACT_COMMAND;
                game_state.menu_coords[MENU_LAYER_ACT_COMMAND as usize] = 0;
            },
            MENU_LAYER_ACT_COMMAND => {
                let act_idx = game_state.menu_coords[MENU_LAYER_ACT_COMMAND as usize] as usize;
                let mut text_to_display = "* You did something.".to_string();
                if let Some(acts) = act_commands_query.iter().next() {
                    if act_idx < acts.commands.len() {
                        let cmd_name = &acts.commands[act_idx];
                        if cmd_name == "Check" {
                            text_to_display = "* FROGGIT - ATK 4 DEF 5\n* Life is difficult for this enemy.".to_string();
                        } else if cmd_name == "Compliment" {
                            text_to_display = "* Froggit didn't understand what you said,\n  but was flattered anyway.".to_string();
                        } else if cmd_name == "Threaten" {
                            text_to_display = "* Froggit didn't understand what you said,\n  but was scared anyway.".to_string();
                        }
                    }
                }
                
                game_state.myfight = 2; 
                game_state.dialog_text = text_to_display.clone();
                
                for entity in menu_items_query.iter() { commands.entity(entity).despawn(); }
                if let Ok((entity, _)) = typewriter_query.get_single_mut() { commands.entity(entity).despawn(); }
                
                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section("", TextStyle { font: asset_server.load("8bitOperatorPlus-Bold.ttf"), font_size: 32.0, color: Color::WHITE }),
                        text_anchor: Anchor::TopLeft,
                        transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        ..default()
                    },
                    Typewriter { full_text: text_to_display, visible_chars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                    MainDialogText,
                    Cleanup,
                ));
            },
            MENU_LAYER_ITEM => {
                let item_index = (game_state.item_page * ITEMS_PER_PAGE) + game_state.menu_coords[MENU_LAYER_ITEM as usize] as usize;
                
                if item_index < game_state.inventory.len() {
                    let item_name = game_state.inventory.remove(item_index);
                    game_state.hp = game_state.max_hp; 
                    let text = format!("* You ate the {}.\n* Your HP was maxed out!", item_name);
                    game_state.myfight = 2; 
                    
                    for entity in menu_items_query.iter() { commands.entity(entity).despawn(); }
                    if let Ok((entity, _)) = typewriter_query.get_single_mut() { commands.entity(entity).despawn(); }
                    
                    commands.spawn((
                        Text2dBundle {
                            text: Text::from_section("", TextStyle { font: asset_server.load("8bitOperatorPlus-Bold.ttf"), font_size: 32.0, color: Color::WHITE }),
                            text_anchor: Anchor::TopLeft,
                            transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                            ..default()
                        },
                        Typewriter { full_text: text, visible_chars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                        MainDialogText,
                        Cleanup,
                    ));
                }
            },
            MENU_LAYER_MERCY => {
                let mercy_idx = game_state.menu_coords[MENU_LAYER_MERCY as usize];
                let text = if mercy_idx == 0 {
                    "* Spare... nothing happened.".to_string()
                } else {
                    "* Escaped...".to_string()
                };
                
                game_state.myfight = 2;
                for entity in menu_items_query.iter() { commands.entity(entity).despawn(); }
                if let Ok((entity, _)) = typewriter_query.get_single_mut() { commands.entity(entity).despawn(); }

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section("", TextStyle { font: asset_server.load("8bitOperatorPlus-Bold.ttf"), font_size: 32.0, color: Color::WHITE }),
                        text_anchor: Anchor::TopLeft,
                        transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        ..default()
                    },
                    Typewriter { full_text: text, visible_chars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                    MainDialogText,
                    Cleanup,
                ));
            },
            _ => {}
        }
    }
    
    if input.just_pressed(KeyCode::KeyX) {
        if layer == MENU_LAYER_FIGHT_TARGET || layer == MENU_LAYER_ACT_TARGET || layer == MENU_LAYER_ITEM || layer == MENU_LAYER_MERCY {
            game_state.menu_layer = MENU_LAYER_TOP;
        } else if layer == MENU_LAYER_ACT_COMMAND {
            game_state.menu_layer = MENU_LAYER_ACT_TARGET;
        }
    }
}
