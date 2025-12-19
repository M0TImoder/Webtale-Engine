use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn menuRenderSystem(
    mut commands: Commands,
    gameState: Res<GameState>,
    gameFonts: Res<GameFonts>,
    menuItems: Query<Entity, With<MenuTextItem>>,
    typewriterQuery: Query<Entity, With<MainDialogText>>,
    actCommandsQuery: Query<&ActCommands, With<EnemyBody>>,
) {
    for entity in menuItems.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if gameState.mnFight != 0 || gameState.myFight != 0 { return; }

    let layer = gameState.menuLayer;

    if layer == MENU_LAYER_TOP {
        if typewriterQuery.is_empty() {
             commands.spawn((
                Text2dBundle {
                    text: Text::from_section("", TextStyle { font: gameFonts.dialog.clone(), font_size: 32.0, color: Color::WHITE }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                    ..default()
                },
                Typewriter { fullText: gameState.dialogText.clone(), visibleChars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                MainDialogText,
                Cleanup,
            ));
        }
    } else {
        if let Ok(entity) = typewriterQuery.get_single() { commands.entity(entity).despawn(); }
        
        let fontStyle = TextStyle { font: gameFonts.dialog.clone(), font_size: 32.0, color: Color::WHITE };
        let startX = 52.0 + 50.0; 
        let startY = 270.0;

        if layer == MENU_LAYER_FIGHT_TARGET || layer == MENU_LAYER_ACT_TARGET {
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("* Froggit", fontStyle.clone()),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(startX, startY) + Vec3::new(0.0, 0.0, Z_TEXT)),
                    ..default()
                },
                MenuTextItem { layer, index: 0 },
                Cleanup,
            ));

            if layer == MENU_LAYER_FIGHT_TARGET {
                let barWidth = 100.0;
                let barHeight = 20.0;
                let barX = startX + 220.0;
                let barY = startY + 5.0;

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite { color: Color::rgb(1.0, 0.0, 0.0), custom_size: Some(Vec2::new(barWidth, barHeight)), anchor: Anchor::TopLeft, ..default() },
                        transform: Transform::from_translation(gml_to_bevy(barX, barY) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        ..default()
                    },
                    MenuTextItem { layer, index: 0 },
                    Cleanup,
                ));

                let hpPercent = (gameState.enemyHp as f32 / gameState.enemyMaxHp as f32).max(0.0);
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite { color: Color::rgb(0.0, 1.0, 0.0), custom_size: Some(Vec2::new(barWidth * hpPercent, barHeight)), anchor: Anchor::TopLeft, ..default() },
                        transform: Transform::from_translation(gml_to_bevy(barX, barY) + Vec3::new(0.0, 0.0, Z_TEXT + 0.1)),
                        ..default()
                    },
                    MenuTextItem { layer, index: 0 },
                    Cleanup,
                ));
            }

        } else if layer == MENU_LAYER_ACT_COMMAND {
            if let Some(acts) = actCommandsQuery.iter().next() {
                for (i, cmdName) in acts.commands.iter().enumerate() {
                    let col = i % 2;
                    let row = i / 2;
                    let xOffset = if col == 0 { 0.0 } else { 240.0 };
                    let yOffset = (row as f32) * 32.0;
                    commands.spawn((
                        Text2dBundle {
                            text: Text::from_section(format!("* {}", cmdName), fontStyle.clone()),
                            text_anchor: Anchor::TopLeft,
                            transform: Transform::from_translation(gml_to_bevy(startX + xOffset, startY + yOffset) + Vec3::new(0.0, 0.0, Z_TEXT)),
                            ..default()
                        },
                        MenuTextItem { layer, index: i as i32 },
                        Cleanup,
                    ));
                }
            }
        } else if layer == MENU_LAYER_ITEM {
            let pageStart = gameState.itemPage * ITEMS_PER_PAGE;
            for i in 0..ITEMS_PER_PAGE {
                if let Some(itemName) = gameState.inventory.get(pageStart + i) {
                    let col = i % 2;
                    let row = i / 2;
                    let xOffset = if col == 0 { 0.0 } else { 240.0 };
                    let yOffset = (row as f32) * 32.0;
                    commands.spawn((
                        Text2dBundle {
                            text: Text::from_section(format!("* {}", itemName), fontStyle.clone()),
                            text_anchor: Anchor::TopLeft,
                            transform: Transform::from_translation(gml_to_bevy(startX + xOffset, startY + yOffset) + Vec3::new(0.0, 0.0, Z_TEXT)),
                            ..default()
                        },
                        MenuTextItem { layer, index: i as i32 },
                        Cleanup,
                    ));
                }
            }
            
            let pageX = startX + 240.0;
            let pageY = startY + 64.0; 
            
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(format!("   PAGE {}", gameState.itemPage + 1), 
                        TextStyle { font: gameFonts.dialog.clone(), font_size: 32.0, color: Color::WHITE }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(pageX, pageY) + Vec3::new(0.0, 0.0, Z_TEXT)),
                    ..default()
                },
                MenuTextItem { layer, index: 99 },
                Cleanup,
            ));

        } else if layer == MENU_LAYER_MERCY {
            let options = ["* Spare", "* Flee"];
            for (i, opt) in options.iter().enumerate() {
                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(*opt, fontStyle.clone()),
                        text_anchor: Anchor::TopLeft,
                        transform: Transform::from_translation(gml_to_bevy(startX, startY + (i as f32 * 32.0)) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        ..default()
                    },
                    MenuTextItem { layer, index: i as i32 },
                    Cleanup,
                ));
            }
        }
    }
}

pub fn updateBoxSize(mut boxRes: ResMut<BattleBox>, time: Res<Time>, _gameState: Res<GameState>) {
    let speed = 15.0 * time.delta_seconds();
    boxRes.current.min.x += (boxRes.target.min.x - boxRes.current.min.x) * speed;
    boxRes.current.min.y += (boxRes.target.min.y - boxRes.current.min.y) * speed;
    boxRes.current.max.x += (boxRes.target.max.x - boxRes.current.max.x) * speed;
    boxRes.current.max.y += (boxRes.target.max.y - boxRes.current.max.y) * speed;
}

pub fn drawBattleBox(
    boxRes: Res<BattleBox>,
    mut border: Query<&mut Transform, (With<BorderVisual>, Without<BackgroundVisual>)>,
    mut borderSpr: Query<&mut Sprite, (With<BorderVisual>, Without<BackgroundVisual>)>,
    mut bg: Query<&mut Transform, (With<BackgroundVisual>, Without<BorderVisual>)>,
    mut bgSpr: Query<&mut Sprite, (With<BackgroundVisual>, Without<BorderVisual>)>,
) {
    let b = &boxRes.current;
    let bevyLeft = ORIGIN_X + b.min.x;
    let bevyRight = ORIGIN_X + b.max.x;
    let bevyTop = ORIGIN_Y - b.min.y; 
    let bevyBottom = ORIGIN_Y - b.max.y;
    let width = bevyRight - bevyLeft;
    let height = bevyTop - bevyBottom;
    let centerX = bevyLeft + width / 2.0;
    let centerY = bevyBottom + height / 2.0;

    if let Ok(mut t) = border.get_single_mut() { t.translation.x = centerX; t.translation.y = centerY; }
    if let Ok(mut s) = borderSpr.get_single_mut() { s.custom_size = Some(Vec2::new(width + 10.0, height + 10.0)); }
    if let Ok(mut t) = bg.get_single_mut() { t.translation.x = centerX; t.translation.y = centerY; }
    if let Ok(mut s) = bgSpr.get_single_mut() { s.custom_size = Some(Vec2::new(width, height)); }
}

pub fn drawUiStatus(
    gameState: Res<GameState>,
    mut redBar: Query<&mut Sprite, (With<HpBarRed>, Without<HpBarYellow>)>,
    mut yelBar: Query<&mut Sprite, (With<HpBarYellow>, Without<HpBarRed>)>,
    mut hpTextQuery: Query<(&mut Text, &mut Transform), (With<HpText>, Without<LvText>)>,
    mut lvTextQuery: Query<&mut Text, (With<LvText>, Without<HpText>)>,
    mut nameTextQuery: Query<&mut Text, (With<PlayerNameText>, Without<HpText>, Without<LvText>)>,
) {
    let barScale = 1.2; let height = 20.0;   
    
    if let Ok(mut s) = redBar.get_single_mut() { s.custom_size = Some(Vec2::new(gameState.maxHp * barScale, height)); }
    if let Ok(mut s) = yelBar.get_single_mut() { s.custom_size = Some(Vec2::new(gameState.hp * barScale, height)); }
    
    if let Ok((mut t, mut trans)) = hpTextQuery.get_single_mut() {
        t.sections[0].value = format!("{:.0} / {:.0}", gameState.hp, gameState.maxHp);
        let visualHpBarX = 250.0;
        let textX = visualHpBarX + (gameState.maxHp * barScale) + 15.0;
        trans.translation = gml_to_bevy(textX, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT);
    }

    if let Ok(mut t) = lvTextQuery.get_single_mut() {
        t.sections[0].value = format!("LV {}", gameState.lv);
    }

    if let Ok(mut t) = nameTextQuery.get_single_mut() {
        t.sections[0].value = gameState.name.clone();
    }
}

pub fn updateButtonSprites(
    gameState: Res<GameState>,
    mut query: Query<(&ButtonVisual, &mut Handle<Image>)>,
) {
    for (btn, mut textureHandle) in query.iter_mut() {
        if gameState.mnFight == 0 && gameState.menuLayer == MENU_LAYER_TOP && btn.index == gameState.menuCoords[MENU_LAYER_TOP as usize] {
            *textureHandle = btn.selectedTexture.clone();
        } else {
            *textureHandle = btn.normalTexture.clone();
        }
    }
}

pub fn animateText(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut gameState: ResMut<GameState>,
    mut query: Query<(Entity, &mut Typewriter, &mut Text)>,
) {
    for (entity, mut writer, mut text) in query.iter_mut() {
        if writer.finished { 
            if gameState.myFight == 2 {
                if input.just_pressed(KeyCode::KeyZ) {
                     commands.entity(entity).despawn();
                     gameState.myFight = 0;
                     gameState.mnFight = 1; 
                     gameState.bubbleTimer.reset(); 
                     gameState.menuLayer = MENU_LAYER_TOP;
                }
            }
            continue; 
        }
        if input.just_pressed(KeyCode::KeyX) {
            writer.visibleChars = writer.fullText.chars().count();
            text.sections[0].value = writer.fullText.clone();
            writer.finished = true; continue;
        }
        if writer.timer.tick(time.delta()).just_finished() {
            let charCount = writer.fullText.chars().count();
            if writer.visibleChars < charCount {
                writer.visibleChars += 1;
                let displayed: String = writer.fullText.chars().take(writer.visibleChars).collect();
                text.sections[0].value = displayed;
            } else { writer.finished = true; }
        }
    }
}

pub fn animateEnemyHead(time: Res<Time>, mut query: Query<(&mut Transform, &mut EnemyHead)>) {
    for (mut transform, mut head) in query.iter_mut() {
        head.timer += time.delta_seconds();
        let offset = (head.timer * 2.0).sin() * 2.0; 
        transform.translation.y = head.baseY + offset;
    }
}
