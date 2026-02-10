use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::window::{MonitorSelection, WindowMode};
use bevy::app::AppExit;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;
use bevy::window::WindowClosed;
use bevy::window::WindowCloseRequested;
use bevy_egui::EguiContexts;
use bevy::sprite::Anchor;

use crate::components::*;
use crate::resources::*;
use crate::constants::*;
use crate::systems::setup::spawn_game_objects;

#[derive(SystemParam)]
pub(crate) struct GlobalInputTextures<'w> {
    images: ResMut<'w, Assets<Image>>,
    editor_preview_texture: ResMut<'w, EditorPreviewTexture>,
    danmaku_preview_texture: ResMut<'w, DanmakuPreviewTexture>,
}

#[derive(SystemParam)]
pub(crate) struct GlobalInputEvents<'w, 's> {
    window_closed_reader: EventReader<'w, 's, WindowClosed>,
    window_close_requested_reader: EventReader<'w, 's, WindowCloseRequested>,
}

// エディタウィンドウ生成
fn spawn_editor_window(
    commands: &mut Commands,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    editor_preview_texture: &mut EditorPreviewTexture,
    danmaku_preview_texture: &mut DanmakuPreviewTexture,
    all_editor_entities: &Query<Entity, With<EditorWindow>>,
) {
    for entity in all_editor_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

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

    editor_preview_texture.0 = image_handle.clone();

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                target: RenderTarget::Image(image_handle.clone()),
                order: -1,
                ..default()
            },
            projection: OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::FixedVertical { viewport_height: 480.0 },
                ..OrthographicProjection::default_2d()
            },
            transform: Transform::from_xyz(0.0, 0.0, 999.9), 
            ..default()
        },
        RenderLayers::layer(0),
        EditorWindow, 
    ));

    let editor_clear_color = Color::srgb_u8(0x22, 0x22, 0x22);
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                target: RenderTarget::Window(WindowRef::Entity(editor_window)),
                clear_color: ClearColorConfig::Custom(editor_clear_color), 
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
            sprite: Sprite { image: image_handle, ..default() },
            transform: Transform::from_xyz(0.0, 75.0, 0.0), 
            ..default()
        },
        RenderLayers::layer(1),
        EditorWindow,
        BattleScreenPreview,
    ));

    let mut preview_image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("Danmaku Preview Texture"),
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
    preview_image.resize(size);
    let preview_image_handle = images.add(preview_image);
    danmaku_preview_texture.0 = preview_image_handle.clone();

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                target: RenderTarget::Image(preview_image_handle.clone()),
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            projection: OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::FixedVertical { viewport_height: 480.0 },
                ..OrthographicProjection::default_2d()
            },
            transform: Transform::from_xyz(0.0, 0.0, 999.9), 
            ..default()
        },
        RenderLayers::layer(2),
        EditorWindow, 
    ));

    let box_center = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 250.0 + (385.0-250.0)/2.0);
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(570.0, 135.0)), 
                ..default()
            },
            transform: Transform::from_translation(box_center + Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        RenderLayers::layer(2),
        EditorWindow,
    ));
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(560.0, 125.0)), 
                ..default()
            },
            transform: Transform::from_translation(box_center + Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        },
        RenderLayers::layer(2),
        EditorWindow,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { image: asset_server.load("texture/heart/spr_heart_0.png"), ..default() },
            transform: Transform::from_translation(box_center + Vec3::new(0.0, 0.0, 2.0)),
            ..default()
        },
        RenderLayers::layer(2),
        EditorWindow,
    ));
}

// エディタ初期生成
pub fn spawn_initial_editor_window(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    all_editor_entities: Query<Entity, With<EditorWindow>>, 
    open_editor_window_query: Query<Entity, (With<EditorWindow>, With<Window>)>, 
    mut images: ResMut<Assets<Image>>,
    mut editor_preview_texture: ResMut<EditorPreviewTexture>,
    mut danmaku_preview_texture: ResMut<DanmakuPreviewTexture>,
) {
    if open_editor_window_query.is_empty() {
        spawn_editor_window(
            &mut commands,
            &asset_server,
            &mut images,
            &mut editor_preview_texture,
            &mut danmaku_preview_texture,
            &all_editor_entities,
        );
    }
}

// グローバル入力
pub fn handle_global_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<(Entity, &mut Window), With<bevy::window::PrimaryWindow>>,
    mut exit_writer: EventWriter<AppExit>,
    asset_server: Res<AssetServer>,
    game_fonts: Res<GameFonts>,
    python_runtime: NonSend<PythonRuntime>,
    mut danmaku_scripts: ResMut<DanmakuScripts>,
    mut menu_render_cache: ResMut<MenuRenderCache>,
    cleanup_query: Query<Entity, With<Cleanup>>,
    all_editor_entities: Query<Entity, With<EditorWindow>>, 
    open_editor_window_query: Query<Entity, (With<EditorWindow>, With<Window>)>, 
    mut egui_contexts: EguiContexts,
    mut textures: GlobalInputTextures,
    mut events: GlobalInputEvents,
    mut game_run_state: ResMut<GameRunState>,
) {
    let mut reset_game = || {
        for entity in cleanup_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        commands.insert_resource(BattleBox {
            current: Rect::new(32.0, 250.0, 602.0, 385.0),
            target: Rect::new(32.0, 250.0, 602.0, 385.0),
        });

        danmaku_scripts.modules.clear();
        danmaku_scripts.rust_specs.clear();
        menu_render_cache.key = None;
        spawn_game_objects(&mut commands, &asset_server, &game_fonts, &python_runtime);
    };

    if game_run_state.reset_requested {
        game_run_state.reset_requested = false;
        reset_game();
    }

    for close_requested in events.window_close_requested_reader.read() {
        if let Ok(editor_window) = open_editor_window_query.get_single() {
            if close_requested.window == editor_window {
                exit_writer.send(AppExit::default());
                return;
            }
        }

        if let Ok((window_entity, mut window)) = window_query.get_single_mut() {
            if close_requested.window == window_entity {
                window.visible = false;
            }
        }
    }

    for closed in events.window_closed_reader.read() {
        if let Ok(editor_window) = open_editor_window_query.get_single() {
            if closed.window == editor_window {
                exit_writer.send(AppExit::default());
                return;
            }
        }
    }

    if input.just_pressed(KeyCode::Escape) {
        exit_writer.send(AppExit::default());
    }

    if (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) && input.just_pressed(KeyCode::Enter) {
        if let Ok((_, mut window)) = window_query.get_single_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                _ => WindowMode::Windowed,
            };
        }
    }

    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) && input.just_pressed(KeyCode::KeyW) {
        if let Ok((_, mut window)) = window_query.get_single_mut() {
            window.visible = true;
        }
    }

    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) && input.just_pressed(KeyCode::KeyR) {
        let mut is_typing = false;
        if let Ok(editor_entity) = open_editor_window_query.get_single() {
            let ctx = egui_contexts.ctx_for_entity_mut(editor_entity);
            if ctx.wants_keyboard_input() {
                is_typing = true;
            }
        }

        if !is_typing {
            reset_game();
        }
    }

    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) && input.just_pressed(KeyCode::KeyE) {
        if open_editor_window_query.is_empty() {
            spawn_editor_window(
                &mut commands,
                &asset_server,
                &mut textures.images,
                &mut textures.editor_preview_texture,
                &mut textures.danmaku_preview_texture,
                &all_editor_entities,
            );
        }
    }
}

// メニュー入力
pub fn menu_input_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<ButtonInput<KeyCode>>,
    mut player_state: ResMut<PlayerState>,
    enemy_state: Res<EnemyState>,
    mut menu_state: ResMut<MenuState>,
    mut combat_state: ResMut<CombatState>,
    mut typewriter_query: Query<(Entity, &mut Typewriter), With<MainDialogText>>,
    act_commands_query: Query<&ActCommands, With<EnemyBody>>,
    menu_items_query: Query<Entity, With<MenuTextItem>>,
    mut egui_contexts: EguiContexts,
    editor_query: Query<Entity, (With<EditorWindow>, With<Window>)>,
    editor_state: Option<Res<EditorState>>,
    item_dict: Res<ItemDictionary>,
){
    if let Ok(editor_entity) = editor_query.get_single() {
        if egui_contexts.ctx_for_entity_mut(editor_entity).wants_keyboard_input() {
            return;
        }
    }

    if let Some(state) = editor_state {
        if state.preview_active {
            return;
        }
    }

    if combat_state.mn_fight != MainFightState::Menu || combat_state.my_fight != MessageFightState::None { return; }
    let layer = menu_state.menu_layer;
    let cursor_idx = menu_state.menu_coords[layer as usize] as usize;
    
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        if layer == MENU_LAYER_TOP {
            menu_state.menu_coords[layer as usize] = (menu_state.menu_coords[layer as usize] - 1 + 4) % 4;
        } else if layer == MENU_LAYER_ACT_COMMAND {
             if cursor_idx % 2 == 1 { menu_state.menu_coords[layer as usize] -= 1; }
        } else if layer == MENU_LAYER_ITEM {
            if cursor_idx % 2 == 1 { 
                menu_state.menu_coords[layer as usize] -= 1; 
            } else if menu_state.item_page > 0 {
                menu_state.item_page -= 1;
                menu_state.menu_coords[layer as usize] += 1; 
            }
        }
    }
    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        if layer == MENU_LAYER_TOP {
            menu_state.menu_coords[layer as usize] = (menu_state.menu_coords[layer as usize] + 1) % 4;
        } else if layer == MENU_LAYER_ACT_COMMAND {
             if cursor_idx % 2 == 0 {
                 if let Some(acts) = act_commands_query.iter().next() {
                     if cursor_idx + 1 < acts.commands.len() { menu_state.menu_coords[layer as usize] += 1; }
                 }
             }
        } else if layer == MENU_LAYER_ITEM {
            let items_on_page = player_state.inventory.len().saturating_sub(menu_state.item_page * ITEMS_PER_PAGE).min(ITEMS_PER_PAGE);
            if cursor_idx % 2 == 0 && cursor_idx + 1 < items_on_page {
                menu_state.menu_coords[layer as usize] += 1;
            } else if cursor_idx % 2 == 1 && (menu_state.item_page + 1) * ITEMS_PER_PAGE < player_state.inventory.len() {
                menu_state.item_page += 1;
                menu_state.menu_coords[layer as usize] -= 1; 
            }
        }
    }
    if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::KeyW) {
         if layer == MENU_LAYER_ACT_COMMAND && cursor_idx >= 2 { menu_state.menu_coords[layer as usize] -= 2; }
         else if layer == MENU_LAYER_ITEM && cursor_idx >= 2 { menu_state.menu_coords[layer as usize] -= 2; }
         else if layer == MENU_LAYER_MERCY && cursor_idx > 0 { menu_state.menu_coords[layer as usize] -= 1; }
    }
    if input.just_pressed(KeyCode::ArrowDown) || input.just_pressed(KeyCode::KeyS) {
         if layer == MENU_LAYER_ACT_COMMAND {
            if let Some(acts) = act_commands_query.iter().next() {
                 if cursor_idx + 2 < acts.commands.len() { menu_state.menu_coords[layer as usize] += 2; }
            }
         } else if layer == MENU_LAYER_ITEM {
            let items_on_page = player_state.inventory.len().saturating_sub(menu_state.item_page * ITEMS_PER_PAGE).min(ITEMS_PER_PAGE);
            if cursor_idx + 2 < items_on_page { menu_state.menu_coords[layer as usize] += 2; }
         } else if layer == MENU_LAYER_MERCY {
             if cursor_idx < 1 { menu_state.menu_coords[layer as usize] += 1; }
         }
    }

    if input.just_pressed(KeyCode::KeyZ) {
        match layer {
            MENU_LAYER_TOP => {
                let selected = menu_state.menu_coords[MENU_LAYER_TOP as usize];
                match selected {
                    0 => { menu_state.menu_layer = MENU_LAYER_FIGHT_TARGET; menu_state.menu_coords[MENU_LAYER_FIGHT_TARGET as usize] = 0; },
                    1 => { menu_state.menu_layer = MENU_LAYER_ACT_TARGET; menu_state.menu_coords[MENU_LAYER_ACT_TARGET as usize] = 0; },
                    2 => { 
                        if !player_state.inventory.is_empty() {
                            menu_state.menu_layer = MENU_LAYER_ITEM; 
                            menu_state.menu_coords[MENU_LAYER_ITEM as usize] = 0; 
                            menu_state.item_page = 0;
                        }
                    }, 
                    3 => { menu_state.menu_layer = MENU_LAYER_MERCY; menu_state.menu_coords[MENU_LAYER_MERCY as usize] = 0; }, 
                    _ => {}
                }
            },
            MENU_LAYER_FIGHT_TARGET => {
                combat_state.last_player_action = "attack".to_string();
                combat_state.last_act_command = None;
                combat_state.mn_fight = MainFightState::PlayerAttackBar; 
                let box_center = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 250.0 + (385.0-250.0)/2.0);
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite { image: asset_server.load("texture/attack/spr_target.png"), custom_size: Some(Vec2::new(566.0, 120.0)), ..default() },
                        transform: Transform::from_translation(box_center + Vec3::new(0.0, 0.0, Z_ATTACK_TARGET)),
                        ..default()
                    },
                    AttackTargetBox,
                    Cleanup,
                ));
                let bar_start_x = gml_to_bevy(32.0, 0.0).x;
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite { image: asset_server.load("texture/attack/spr_targetchoice_1.png"), custom_size: Some(Vec2::new(14.0, 120.0)), ..default() },
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
                menu_state.menu_layer = MENU_LAYER_ACT_COMMAND;
                menu_state.menu_coords[MENU_LAYER_ACT_COMMAND as usize] = 0;
            },
            MENU_LAYER_ACT_COMMAND => {
                let act_idx = menu_state.menu_coords[MENU_LAYER_ACT_COMMAND as usize] as usize;
                let mut text_to_display = "* You did something.".to_string();
                if let Some(acts) = act_commands_query.iter().next() {
                    if act_idx < acts.commands.len() {
                        let cmd_name = &acts.commands[act_idx];
                        combat_state.last_player_action = "act".to_string();
                        combat_state.last_act_command = Some(cmd_name.clone());
                        if let Some(text) = enemy_state.act_texts.get(cmd_name) {
                            text_to_display = text.clone();
                        } else if cmd_name == "Check" {
                            let enemy_name = if enemy_state.name.is_empty() { "ENEMY".to_string() } else { enemy_state.name.to_uppercase() };
                            text_to_display = format!(
                                "* {} - ATK {} DEF {}\n* ...",
                                enemy_name,
                                enemy_state.atk,
                                enemy_state.def
                            );
                        }
                    }
                }
                
                combat_state.my_fight = MessageFightState::PlayerActionText; 
                menu_state.dialog_text = text_to_display.clone();
                
                for entity in menu_items_query.iter() { commands.entity(entity).despawn(); }
                if let Ok((entity, _)) = typewriter_query.get_single_mut() { commands.entity(entity).despawn(); }
                
                commands.spawn((
                    Text2d::new(""),
                    TextFont { font: asset_server.load("font/8bitOperatorPlus-Bold.ttf"), font_size: 32.0 * TEXT_SCALE, ..default() },
                    TextColor(Color::WHITE),
                    Anchor::TopLeft,
                    Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                    Typewriter { full_text: text_to_display, visible_chars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                    MainDialogText,
                    Cleanup,
                ));
            },
            MENU_LAYER_ITEM => {
                let item_index = (menu_state.item_page * ITEMS_PER_PAGE) + menu_state.menu_coords[MENU_LAYER_ITEM as usize] as usize;
                
                if item_index < player_state.inventory.len() {
                    combat_state.last_player_action = "item".to_string();
                    combat_state.last_act_command = None;
                    let item_name = player_state.inventory.remove(item_index);
                    
                    let (heal_amount, flavor_text) = if let Some(info) = item_dict.0.get(&item_name) {
                        (info.heal_amount, info.text.clone())
                    } else {
                        (0, "...".to_string())
                    };

                    let old_hp = player_state.hp;
                    player_state.hp = (player_state.hp + heal_amount as f32).min(player_state.max_hp);
                    let recovered = (player_state.hp - old_hp) as i32;

                    let mut text = format!("* You ate the {}.", item_name);

                    if player_state.hp >= player_state.max_hp {
                        text.push_str("\n* Your HP was maxed out!");
                    } else {
                        if !flavor_text.is_empty() {
                            text.push_str(&format!("\n* {}", flavor_text));
                            text.push_str(&format!("\n* You recovered {} HP!", recovered));
                        } else {
                            text.push_str(&format!("\n* You recovered {} HP!", recovered));
                        }
                    }

                    combat_state.my_fight = MessageFightState::PlayerActionText; 
                    
                    for entity in menu_items_query.iter() { commands.entity(entity).despawn(); }
                    if let Ok((entity, _)) = typewriter_query.get_single_mut() { commands.entity(entity).despawn(); }
                    
                    commands.spawn((
                        Text2d::new(""),
                        TextFont { font: asset_server.load("font/8bitOperatorPlus-Bold.ttf"), font_size: 32.0 * TEXT_SCALE, ..default() },
                        TextColor(Color::WHITE),
                        Anchor::TopLeft,
                        Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        Typewriter { full_text: text, visible_chars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                        MainDialogText,
                        Cleanup,
                    ));
                }
            },
            MENU_LAYER_MERCY => {
                let mercy_idx = menu_state.menu_coords[MENU_LAYER_MERCY as usize];
                combat_state.last_act_command = None;
                combat_state.last_player_action = if mercy_idx == 0 {
                    "spare".to_string()
                } else {
                    "flee".to_string()
                };
                let text = if mercy_idx == 0 {
                    "* Spare... nothing happened.".to_string()
                } else {
                    "* Escaped...".to_string()
                };
                
                combat_state.my_fight = MessageFightState::PlayerActionText;
                for entity in menu_items_query.iter() { commands.entity(entity).despawn(); }
                if let Ok((entity, _)) = typewriter_query.get_single_mut() { commands.entity(entity).despawn(); }

                commands.spawn((
                    Text2d::new(""),
                    TextFont { font: asset_server.load("font/8bitOperatorPlus-Bold.ttf"), font_size: 32.0 * TEXT_SCALE, ..default() },
                    TextColor(Color::WHITE),
                    Anchor::TopLeft,
                    Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
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
            menu_state.menu_layer = MENU_LAYER_TOP;
        } else if layer == MENU_LAYER_ACT_COMMAND {
            menu_state.menu_layer = MENU_LAYER_ACT_TARGET;
        }
    }
}
