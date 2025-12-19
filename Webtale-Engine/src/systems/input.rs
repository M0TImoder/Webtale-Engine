use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::app::AppExit;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;
use bevy_egui::EguiContexts;
use bevy::sprite::Anchor;

use crate::components::*;
use crate::resources::*;
use crate::constants::*;
use crate::systems::setup::spawnGameObjects; 

pub fn handleGlobalInput(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut windowQuery: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    mut exitWriter: EventWriter<AppExit>,
    assetServer: Res<AssetServer>,
    gameFonts: Res<GameFonts>,
    cleanupQuery: Query<Entity, With<Cleanup>>,
    allEditorEntities: Query<Entity, With<EditorWindow>>, 
    openEditorWindowQuery: Query<Entity, (With<EditorWindow>, With<Window>)>, 
    mut images: ResMut<Assets<Image>>,
    mut eguiContexts: EguiContexts,
    mut editorPreviewTexture: ResMut<EditorPreviewTexture>,
    mut danmakuPreviewTexture: ResMut<DanmakuPreviewTexture>,
) {
    if input.just_pressed(KeyCode::Escape) {
        exitWriter.send(AppExit::default());
    }

    if (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) && input.just_pressed(KeyCode::Enter) {
        if let Ok(mut window) = windowQuery.get_single_mut() {
             window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen,
                _ => WindowMode::Windowed,
            };
        }
    }

    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) && input.just_pressed(KeyCode::KeyR) {
        let mut isTyping = false;
        if let Ok(editorEntity) = openEditorWindowQuery.get_single() {
            let ctx = eguiContexts.ctx_for_window_mut(editorEntity);
            if ctx.wants_keyboard_input() {
                isTyping = true;
            }
        }

        if !isTyping {
            for entity in cleanupQuery.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
            commands.insert_resource(BattleBox {
                current: Rect::new(32.0, 250.0, 602.0, 385.0),
                target: Rect::new(32.0, 250.0, 602.0, 385.0),
            });

            spawnGameObjects(&mut commands, &assetServer, &gameFonts);
        }
    }

    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) && input.just_pressed(KeyCode::KeyE) {
        if openEditorWindowQuery.is_empty() {
            for entity in allEditorEntities.iter() {
                commands.entity(entity).despawn_recursive();
            }

            let editorWindow = commands.spawn((
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
            let imageHandle = images.add(image);

            editorPreviewTexture.0 = imageHandle.clone();

            commands.spawn((
                Camera2dBundle {
                    camera: Camera {
                        target: RenderTarget::Image(imageHandle.clone()),
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
                        target: RenderTarget::Window(WindowRef::Entity(editorWindow)),
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
                    texture: imageHandle,
                    transform: Transform::from_xyz(0.0, 75.0, 0.0), 
                    ..default()
                },
                RenderLayers::layer(1),
                EditorWindow,
                BattleScreenPreview,
            ));

            let mut previewImage = Image {
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
            previewImage.resize(size);
            let previewImageHandle = images.add(previewImage);
            danmakuPreviewTexture.0 = previewImageHandle.clone();

            commands.spawn((
                Camera2dBundle {
                    camera: Camera {
                        target: RenderTarget::Image(previewImageHandle.clone()),
                        order: -1,
                        clear_color: ClearColorConfig::Custom(Color::BLACK),
                        ..default()
                    },
                    projection: OrthographicProjection {
                        scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(480.0),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 999.9), 
                    ..default()
                },
                RenderLayers::layer(2),
                EditorWindow, 
            ));

            let boxCenter = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 250.0 + (385.0-250.0)/2.0);
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::new(570.0, 135.0)), 
                        ..default()
                    },
                    transform: Transform::from_translation(boxCenter + Vec3::new(0.0, 0.0, 0.0)),
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
                    transform: Transform::from_translation(boxCenter + Vec3::new(0.0, 0.0, 1.0)),
                    ..default()
                },
                RenderLayers::layer(2),
                EditorWindow,
            ));

            commands.spawn((
                SpriteBundle {
                    texture: assetServer.load("player/spr_soul_0.png"),
                    transform: Transform::from_translation(boxCenter + Vec3::new(0.0, 0.0, 2.0)),
                    ..default()
                },
                RenderLayers::layer(2),
                EditorWindow,
            ));
        }
    }
}

pub fn menuInputSystem(
    mut commands: Commands,
    assetServer: Res<AssetServer>,
    input: Res<ButtonInput<KeyCode>>,
    mut gameState: ResMut<GameState>,
    mut typewriterQuery: Query<(Entity, &mut Typewriter), With<MainDialogText>>,
    actCommandsQuery: Query<&ActCommands, With<EnemyBody>>,
    menuItemsQuery: Query<Entity, With<MenuTextItem>>,
    mut eguiContexts: EguiContexts,
    editorQuery: Query<Entity, (With<EditorWindow>, With<Window>)>,
    editorState: Option<Res<EditorState>>,
    itemDict: Res<ItemDictionary>,
) {
    if let Ok(editorEntity) = editorQuery.get_single() {
        if eguiContexts.ctx_for_window_mut(editorEntity).wants_keyboard_input() {
            return;
        }
    }

    if let Some(state) = editorState {
        if state.currentTab == EditorTab::DanmakuPreview {
            return;
        }
    }

    if gameState.mnFight != 0 || gameState.myFight != 0 { return; }
    let layer = gameState.menuLayer;
    let cursorIdx = gameState.menuCoords[layer as usize] as usize;
    
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        if layer == MENU_LAYER_TOP {
            gameState.menuCoords[layer as usize] = (gameState.menuCoords[layer as usize] - 1 + 4) % 4;
        } else if layer == MENU_LAYER_ACT_COMMAND {
             if cursorIdx % 2 == 1 { gameState.menuCoords[layer as usize] -= 1; }
        } else if layer == MENU_LAYER_ITEM {
            if cursorIdx % 2 == 1 { 
                gameState.menuCoords[layer as usize] -= 1; 
            } else if gameState.itemPage > 0 {
                gameState.itemPage -= 1;
                gameState.menuCoords[layer as usize] += 1; 
            }
        }
    }
    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        if layer == MENU_LAYER_TOP {
            gameState.menuCoords[layer as usize] = (gameState.menuCoords[layer as usize] + 1) % 4;
        } else if layer == MENU_LAYER_ACT_COMMAND {
             if cursorIdx % 2 == 0 {
                 if let Some(acts) = actCommandsQuery.iter().next() {
                     if cursorIdx + 1 < acts.commands.len() { gameState.menuCoords[layer as usize] += 1; }
                 }
             }
        } else if layer == MENU_LAYER_ITEM {
            let itemsOnPage = gameState.inventory.len().saturating_sub(gameState.itemPage * ITEMS_PER_PAGE).min(ITEMS_PER_PAGE);
            if cursorIdx % 2 == 0 && cursorIdx + 1 < itemsOnPage {
                gameState.menuCoords[layer as usize] += 1;
            } else if cursorIdx % 2 == 1 && (gameState.itemPage + 1) * ITEMS_PER_PAGE < gameState.inventory.len() {
                gameState.itemPage += 1;
                gameState.menuCoords[layer as usize] -= 1; 
            }
        }
    }
    if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::KeyW) {
         if layer == MENU_LAYER_ACT_COMMAND && cursorIdx >= 2 { gameState.menuCoords[layer as usize] -= 2; }
         else if layer == MENU_LAYER_ITEM && cursorIdx >= 2 { gameState.menuCoords[layer as usize] -= 2; }
         else if layer == MENU_LAYER_MERCY && cursorIdx > 0 { gameState.menuCoords[layer as usize] -= 1; }
    }
    if input.just_pressed(KeyCode::ArrowDown) || input.just_pressed(KeyCode::KeyS) {
         if layer == MENU_LAYER_ACT_COMMAND {
            if let Some(acts) = actCommandsQuery.iter().next() {
                 if cursorIdx + 2 < acts.commands.len() { gameState.menuCoords[layer as usize] += 2; }
            }
         } else if layer == MENU_LAYER_ITEM {
            let itemsOnPage = gameState.inventory.len().saturating_sub(gameState.itemPage * ITEMS_PER_PAGE).min(ITEMS_PER_PAGE);
            if cursorIdx + 2 < itemsOnPage { gameState.menuCoords[layer as usize] += 2; }
         } else if layer == MENU_LAYER_MERCY {
             if cursorIdx < 1 { gameState.menuCoords[layer as usize] += 1; }
         }
    }

    if input.just_pressed(KeyCode::KeyZ) {
        match layer {
            MENU_LAYER_TOP => {
                let selected = gameState.menuCoords[MENU_LAYER_TOP as usize];
                match selected {
                    0 => { gameState.menuLayer = MENU_LAYER_FIGHT_TARGET; gameState.menuCoords[MENU_LAYER_FIGHT_TARGET as usize] = 0; },
                    1 => { gameState.menuLayer = MENU_LAYER_ACT_TARGET; gameState.menuCoords[MENU_LAYER_ACT_TARGET as usize] = 0; },
                    2 => { 
                        if !gameState.inventory.is_empty() {
                            gameState.menuLayer = MENU_LAYER_ITEM; 
                            gameState.menuCoords[MENU_LAYER_ITEM as usize] = 0; 
                            gameState.itemPage = 0;
                        }
                    }, 
                    3 => { gameState.menuLayer = MENU_LAYER_MERCY; gameState.menuCoords[MENU_LAYER_MERCY as usize] = 0; }, 
                    _ => {}
                }
            },
            MENU_LAYER_FIGHT_TARGET => {
                gameState.mnFight = 4; 
                let boxCenter = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 250.0 + (385.0-250.0)/2.0);
                commands.spawn((
                    SpriteBundle {
                        texture: assetServer.load("attack/spr_target.png"),
                        sprite: Sprite { custom_size: Some(Vec2::new(566.0, 120.0)), ..default() },
                        transform: Transform::from_translation(boxCenter + Vec3::new(0.0, 0.0, Z_ATTACK_TARGET)),
                        ..default()
                    },
                    AttackTargetBox,
                    Cleanup,
                ));
                let barStartX = gml_to_bevy(32.0, 0.0).x;
                commands.spawn((
                    SpriteBundle {
                        texture: assetServer.load("attack/spr_targetchoice_1.png"),
                        sprite: Sprite { custom_size: Some(Vec2::new(14.0, 120.0)), ..default() },
                        transform: Transform::from_translation(Vec3::new(barStartX, boxCenter.y, Z_ATTACK_BAR)),
                        ..default()
                    },
                    AttackBar { speed: 420.0, moving: true, flashTimer: Timer::from_seconds(0.08, TimerMode::Repeating), flashState: true },
                    Cleanup,
                ));

                for entity in menuItemsQuery.iter() { commands.entity(entity).despawn(); }
                if let Ok((entity, _)) = typewriterQuery.get_single_mut() { commands.entity(entity).despawn(); }
            },
            MENU_LAYER_ACT_TARGET => {
                gameState.menuLayer = MENU_LAYER_ACT_COMMAND;
                gameState.menuCoords[MENU_LAYER_ACT_COMMAND as usize] = 0;
            },
            MENU_LAYER_ACT_COMMAND => {
                let actIdx = gameState.menuCoords[MENU_LAYER_ACT_COMMAND as usize] as usize;
                let mut textToDisplay = "* You did something.".to_string();
                if let Some(acts) = actCommandsQuery.iter().next() {
                    if actIdx < acts.commands.len() {
                        let cmdName = &acts.commands[actIdx];
                        if cmdName == "Check" {
                            textToDisplay = "* FROGGIT - ATK 4 DEF 5\n* Life is difficult for this enemy.".to_string();
                        } else if cmdName == "Compliment" {
                            textToDisplay = "* Froggit didn't understand what you said,\n  but was flattered anyway.".to_string();
                        } else if cmdName == "Threaten" {
                            textToDisplay = "* Froggit didn't understand what you said,\n  but was scared anyway.".to_string();
                        }
                    }
                }
                
                gameState.myFight = 2; 
                gameState.dialogText = textToDisplay.clone();
                
                for entity in menuItemsQuery.iter() { commands.entity(entity).despawn(); }
                if let Ok((entity, _)) = typewriterQuery.get_single_mut() { commands.entity(entity).despawn(); }
                
                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section("", TextStyle { font: assetServer.load("font/8bitOperatorPlus-Bold.ttf"), font_size: 32.0, color: Color::WHITE }),
                        text_anchor: Anchor::TopLeft,
                        transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        ..default()
                    },
                    Typewriter { fullText: textToDisplay, visibleChars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                    MainDialogText,
                    Cleanup,
                ));
            },
            MENU_LAYER_ITEM => {
                let itemIndex = (gameState.itemPage * ITEMS_PER_PAGE) + gameState.menuCoords[MENU_LAYER_ITEM as usize] as usize;
                
                if itemIndex < gameState.inventory.len() {
                    let itemName = gameState.inventory.remove(itemIndex);
                    
                    let (healAmount, flavorText) = if let Some(info) = itemDict.0.get(&itemName) {
                        (info.healAmount, info.text.clone())
                    } else {
                        (0, "...".to_string())
                    };

                    let oldHp = gameState.hp;
                    gameState.hp = (gameState.hp + healAmount as f32).min(gameState.maxHp);
                    let recovered = (gameState.hp - oldHp) as i32;

                    let mut text = format!("* You ate the {}.", itemName);

                    if gameState.hp >= gameState.maxHp {
                        text.push_str("\n* Your HP was maxed out!");
                    } else {
                        if !flavorText.is_empty() {
                            text.push_str(&format!("\n* {}", flavorText));
                            text.push_str(&format!("\n* You recovered {} HP!", recovered));
                        } else {
                            text.push_str(&format!("\n* You recovered {} HP!", recovered));
                        }
                    }

                    gameState.myFight = 2; 
                    
                    for entity in menuItemsQuery.iter() { commands.entity(entity).despawn(); }
                    if let Ok((entity, _)) = typewriterQuery.get_single_mut() { commands.entity(entity).despawn(); }
                    
                    commands.spawn((
                        Text2dBundle {
                            text: Text::from_section("", TextStyle { font: assetServer.load("font/8bitOperatorPlus-Bold.ttf"), font_size: 32.0, color: Color::WHITE }),
                            text_anchor: Anchor::TopLeft,
                            transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                            ..default()
                        },
                        Typewriter { fullText: text, visibleChars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                        MainDialogText,
                        Cleanup,
                    ));
                }
            },
            MENU_LAYER_MERCY => {
                let mercyIdx = gameState.menuCoords[MENU_LAYER_MERCY as usize];
                let text = if mercyIdx == 0 {
                    "* Spare... nothing happened.".to_string()
                } else {
                    "* Escaped...".to_string()
                };
                
                gameState.myFight = 2;
                for entity in menuItemsQuery.iter() { commands.entity(entity).despawn(); }
                if let Ok((entity, _)) = typewriterQuery.get_single_mut() { commands.entity(entity).despawn(); }

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section("", TextStyle { font: assetServer.load("font/8bitOperatorPlus-Bold.ttf"), font_size: 32.0, color: Color::WHITE }),
                        text_anchor: Anchor::TopLeft,
                        transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        ..default()
                    },
                    Typewriter { fullText: text, visibleChars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                    MainDialogText,
                    Cleanup,
                ));
            },
            _ => {}
        }
    }
    
    if input.just_pressed(KeyCode::KeyX) {
        if layer == MENU_LAYER_FIGHT_TARGET || layer == MENU_LAYER_ACT_TARGET || layer == MENU_LAYER_ITEM || layer == MENU_LAYER_MERCY {
            gameState.menuLayer = MENU_LAYER_TOP;
        } else if layer == MENU_LAYER_ACT_COMMAND {
            gameState.menuLayer = MENU_LAYER_ACT_TARGET;
        }
    }
}
