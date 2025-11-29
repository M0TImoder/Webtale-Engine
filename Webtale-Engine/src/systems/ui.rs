use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn menu_render_system(
    mut commands: Commands,
    game_state: Res<GameState>,
    game_fonts: Res<GameFonts>,
    menu_items: Query<Entity, With<MenuTextItem>>,
    typewriter_query: Query<Entity, With<MainDialogText>>,
    act_commands_query: Query<&ActCommands, With<EnemyBody>>,
) {
    for entity in menu_items.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if game_state.mnfight != 0 || game_state.myfight != 0 { return; }

    let layer = game_state.menu_layer;

    if layer == MENU_LAYER_TOP {
        if typewriter_query.is_empty() {
             commands.spawn((
                Text2dBundle {
                    text: Text::from_section("", TextStyle { font: game_fonts.dialog.clone(), font_size: 32.0, color: Color::WHITE }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                    ..default()
                },
                Typewriter { full_text: game_state.dialog_text.clone(), visible_chars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                MainDialogText,
                Cleanup,
            ));
        }
    } else {
        if let Ok(entity) = typewriter_query.get_single() { commands.entity(entity).despawn(); }
        
        let font_style = TextStyle { font: game_fonts.dialog.clone(), font_size: 32.0, color: Color::WHITE };
        let start_x = 52.0 + 50.0; 
        let start_y = 270.0;

        if layer == MENU_LAYER_FIGHT_TARGET || layer == MENU_LAYER_ACT_TARGET {
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("* Froggit", font_style.clone()),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(start_x, start_y) + Vec3::new(0.0, 0.0, Z_TEXT)),
                    ..default()
                },
                MenuTextItem { layer, index: 0 },
                Cleanup,
            ));

            if layer == MENU_LAYER_FIGHT_TARGET {
                let bar_width = 100.0;
                let bar_height = 20.0;
                let bar_x = start_x + 220.0;
                let bar_y = start_y + 5.0;

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite { color: Color::rgb(1.0, 0.0, 0.0), custom_size: Some(Vec2::new(bar_width, bar_height)), anchor: Anchor::TopLeft, ..default() },
                        transform: Transform::from_translation(gml_to_bevy(bar_x, bar_y) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        ..default()
                    },
                    MenuTextItem { layer, index: 0 },
                    Cleanup,
                ));

                let hp_percent = (game_state.enemy_hp as f32 / game_state.enemy_max_hp as f32).max(0.0);
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite { color: Color::rgb(0.0, 1.0, 0.0), custom_size: Some(Vec2::new(bar_width * hp_percent, bar_height)), anchor: Anchor::TopLeft, ..default() },
                        transform: Transform::from_translation(gml_to_bevy(bar_x, bar_y) + Vec3::new(0.0, 0.0, Z_TEXT + 0.1)),
                        ..default()
                    },
                    MenuTextItem { layer, index: 0 },
                    Cleanup,
                ));
            }

        } else if layer == MENU_LAYER_ACT_COMMAND {
            if let Some(acts) = act_commands_query.iter().next() {
                for (i, cmd_name) in acts.commands.iter().enumerate() {
                    let col = i % 2;
                    let row = i / 2;
                    let x_offset = if col == 0 { 0.0 } else { 240.0 };
                    let y_offset = (row as f32) * 32.0;
                    commands.spawn((
                        Text2dBundle {
                            text: Text::from_section(format!("* {}", cmd_name), font_style.clone()),
                            text_anchor: Anchor::TopLeft,
                            transform: Transform::from_translation(gml_to_bevy(start_x + x_offset, start_y + y_offset) + Vec3::new(0.0, 0.0, Z_TEXT)),
                            ..default()
                        },
                        MenuTextItem { layer, index: i as i32 },
                        Cleanup,
                    ));
                }
            }
        } else if layer == MENU_LAYER_ITEM {
            let page_start = game_state.item_page * ITEMS_PER_PAGE;
            for i in 0..ITEMS_PER_PAGE {
                if let Some(item_name) = game_state.inventory.get(page_start + i) {
                    let col = i % 2;
                    let row = i / 2;
                    let x_offset = if col == 0 { 0.0 } else { 240.0 };
                    let y_offset = (row as f32) * 32.0;
                    commands.spawn((
                        Text2dBundle {
                            text: Text::from_section(format!("* {}", item_name), font_style.clone()),
                            text_anchor: Anchor::TopLeft,
                            transform: Transform::from_translation(gml_to_bevy(start_x + x_offset, start_y + y_offset) + Vec3::new(0.0, 0.0, Z_TEXT)),
                            ..default()
                        },
                        MenuTextItem { layer, index: i as i32 },
                        Cleanup,
                    ));
                }
            }
            
            let page_x = start_x + 240.0;
            let page_y = start_y + 64.0; 
            
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(format!("   PAGE {}", game_state.item_page + 1), 
                        TextStyle { font: game_fonts.dialog.clone(), font_size: 32.0, color: Color::WHITE }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(page_x, page_y) + Vec3::new(0.0, 0.0, Z_TEXT)),
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
                        text: Text::from_section(*opt, font_style.clone()),
                        text_anchor: Anchor::TopLeft,
                        transform: Transform::from_translation(gml_to_bevy(start_x, start_y + (i as f32 * 32.0)) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        ..default()
                    },
                    MenuTextItem { layer, index: i as i32 },
                    Cleanup,
                ));
            }
        }
    }
}

pub fn update_box_size(mut box_res: ResMut<BattleBox>, time: Res<Time>, _game_state: Res<GameState>) {
    let speed = 15.0 * time.delta_seconds();
    box_res.current.min.x += (box_res.target.min.x - box_res.current.min.x) * speed;
    box_res.current.min.y += (box_res.target.min.y - box_res.current.min.y) * speed;
    box_res.current.max.x += (box_res.target.max.x - box_res.current.max.x) * speed;
    box_res.current.max.y += (box_res.target.max.y - box_res.current.max.y) * speed;
}

pub fn draw_battle_box(
    box_res: Res<BattleBox>,
    mut border: Query<&mut Transform, (With<BorderVisual>, Without<BackgroundVisual>)>,
    mut border_spr: Query<&mut Sprite, (With<BorderVisual>, Without<BackgroundVisual>)>,
    mut bg: Query<&mut Transform, (With<BackgroundVisual>, Without<BorderVisual>)>,
    mut bg_spr: Query<&mut Sprite, (With<BackgroundVisual>, Without<BorderVisual>)>,
) {
    let b = &box_res.current;
    let bevy_left = ORIGIN_X + b.min.x;
    let bevy_right = ORIGIN_X + b.max.x;
    let bevy_top = ORIGIN_Y - b.min.y; 
    let bevy_bottom = ORIGIN_Y - b.max.y;
    let width = bevy_right - bevy_left;
    let height = bevy_top - bevy_bottom;
    let center_x = bevy_left + width / 2.0;
    let center_y = bevy_bottom + height / 2.0;

    if let Ok(mut t) = border.get_single_mut() { t.translation.x = center_x; t.translation.y = center_y; }
    if let Ok(mut s) = border_spr.get_single_mut() { s.custom_size = Some(Vec2::new(width + 10.0, height + 10.0)); }
    if let Ok(mut t) = bg.get_single_mut() { t.translation.x = center_x; t.translation.y = center_y; }
    if let Ok(mut s) = bg_spr.get_single_mut() { s.custom_size = Some(Vec2::new(width, height)); }
}

pub fn draw_ui_status(
    game_state: Res<GameState>,
    mut red_bar: Query<&mut Sprite, (With<HpBarRed>, Without<HpBarYellow>)>,
    mut yel_bar: Query<&mut Sprite, (With<HpBarYellow>, Without<HpBarRed>)>,
    mut text: Query<(&mut Text, &mut Transform), With<HpText>>,
) {
    let bar_scale = 1.2; let height = 20.0;   
    if let Ok(mut s) = red_bar.get_single_mut() { s.custom_size = Some(Vec2::new(game_state.max_hp * bar_scale, height)); }
    if let Ok(mut s) = yel_bar.get_single_mut() { s.custom_size = Some(Vec2::new(game_state.hp * bar_scale, height)); }
    if let Ok((mut t, mut trans)) = text.get_single_mut() {
        t.sections[0].value = format!("{:.0} / {:.0}", game_state.hp, game_state.max_hp);
        let visual_hp_bar_x = 250.0;
        let text_x = visual_hp_bar_x + (game_state.max_hp * bar_scale) + 15.0;
        trans.translation = gml_to_bevy(text_x, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT);
    }
}

pub fn update_button_sprites(
    game_state: Res<GameState>,
    mut query: Query<(&ButtonVisual, &mut Handle<Image>)>,
) {
    for (btn, mut texture_handle) in query.iter_mut() {
        if game_state.mnfight == 0 && game_state.menu_layer == MENU_LAYER_TOP && btn.index == game_state.menu_coords[MENU_LAYER_TOP as usize] {
            *texture_handle = btn.selected_texture.clone();
        } else {
            *texture_handle = btn.normal_texture.clone();
        }
    }
}

pub fn animate_text(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut query: Query<(Entity, &mut Typewriter, &mut Text)>,
) {
    for (entity, mut writer, mut text) in query.iter_mut() {
        if writer.finished { 
            if game_state.myfight == 2 {
                if input.just_pressed(KeyCode::KeyZ) {
                     commands.entity(entity).despawn();
                     game_state.myfight = 0;
                     game_state.mnfight = 1; 
                     game_state.bubble_timer.reset(); 
                     game_state.menu_layer = MENU_LAYER_TOP;
                }
            }
            continue; 
        }
        if input.just_pressed(KeyCode::KeyX) {
            writer.visible_chars = writer.full_text.chars().count();
            text.sections[0].value = writer.full_text.clone();
            writer.finished = true; continue;
        }
        if writer.timer.tick(time.delta()).just_finished() {
            let char_count = writer.full_text.chars().count();
            if writer.visible_chars < char_count {
                writer.visible_chars += 1;
                let displayed: String = writer.full_text.chars().take(writer.visible_chars).collect();
                text.sections[0].value = displayed;
            } else { writer.finished = true; }
        }
    }
}

pub fn animate_enemy_head(time: Res<Time>, mut query: Query<(&mut Transform, &mut EnemyHead)>) {
    for (mut transform, mut head) in query.iter_mut() {
        head.timer += time.delta_seconds();
        let offset = (head.timer * 2.0).sin() * 2.0; 
        transform.translation.y = head.base_y + offset;
    }
}
