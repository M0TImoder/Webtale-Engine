use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

// メニュー描画
pub fn menu_render_system(
    mut commands: Commands,
    combat_state: Res<CombatState>,
    enemy_state: Res<EnemyState>,
    menu_state: Res<MenuState>,
    player_state: Res<PlayerState>,
    game_fonts: Res<GameFonts>,
    menu_items: Query<Entity, With<MenuTextItem>>,
    typewriter_query: Query<Entity, With<MainDialogText>>,
    act_commands_query: Query<&ActCommands, With<EnemyBody>>,
    mut menu_render_cache: ResMut<MenuRenderCache>,
){
    let is_menu = combat_state.mn_fight == MainFightState::Menu && combat_state.my_fight == MessageFightState::None;
    if !is_menu {
        if menu_render_cache.key.is_some() {
            for entity in menu_items.iter() {
                commands.entity(entity).despawn_recursive();
            }
            menu_render_cache.key = None;
        }
        return;
    }

    // メニュー差分
    let act_commands = act_commands_query
        .iter()
        .next()
        .map(|acts| acts.commands.clone())
        .unwrap_or_default();
    let key = MenuRenderKey {
        menu_layer: menu_state.menu_layer,
        menu_coords: menu_state.menu_coords.clone(),
        item_page: menu_state.item_page,
        dialog_text: menu_state.dialog_text.clone(),
        enemy_name: enemy_state.name.clone(),
        enemy_hp: enemy_state.hp,
        enemy_max_hp: enemy_state.max_hp,
        act_commands,
        inventory: player_state.inventory.clone(),
    };

    let menu_font = TextFont { font: game_fonts.dialog.clone(), font_size: 32.0 * TEXT_SCALE, ..default() };
    let menu_color = TextColor(Color::WHITE);

    if menu_render_cache.key.as_ref() == Some(&key) {
        if menu_state.menu_layer == MENU_LAYER_TOP && typewriter_query.is_empty() {
            commands.spawn((
                Text2d::new(""),
                menu_font.clone(),
                menu_color,
                Anchor::TopLeft,
                Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                Typewriter { full_text: menu_state.dialog_text.clone(), visible_chars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                MainDialogText,
                Cleanup,
            ));
        }
        return;
    }
    menu_render_cache.key = Some(key);

    for entity in menu_items.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let layer = menu_state.menu_layer;

    if layer == MENU_LAYER_TOP {
        if typewriter_query.is_empty() {
             commands.spawn((
                Text2d::new(""),
                menu_font.clone(),
                menu_color,
                Anchor::TopLeft,
                Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
                Typewriter { full_text: menu_state.dialog_text.clone(), visible_chars: 0, timer: Timer::from_seconds(0.03, TimerMode::Repeating), finished: false },
                MainDialogText,
                Cleanup,
            ));
        }
    } else {
        if let Ok(entity) = typewriter_query.get_single() { commands.entity(entity).despawn(); }
        
        let start_x = 52.0 + 50.0; 
        let start_y = 270.0;

        if layer == MENU_LAYER_FIGHT_TARGET || layer == MENU_LAYER_ACT_TARGET {
            let enemy_name = if enemy_state.name.is_empty() { "Enemy" } else { &enemy_state.name };
            commands.spawn((
                Text2d::new(format!("* {}", enemy_name)),
                menu_font.clone(),
                menu_color,
                Anchor::TopLeft,
                Transform::from_translation(gml_to_bevy(start_x, start_y) + Vec3::new(0.0, 0.0, Z_TEXT)),
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

                let hp_percent = (enemy_state.hp as f32 / enemy_state.max_hp as f32).max(0.0);
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
                        Text2d::new(format!("* {}", cmd_name)),
                        menu_font.clone(),
                        menu_color,
                        Anchor::TopLeft,
                        Transform::from_translation(gml_to_bevy(start_x + x_offset, start_y + y_offset) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        MenuTextItem { layer, index: i as i32 },
                        Cleanup,
                    ));
                }
            }
        } else if layer == MENU_LAYER_ITEM {
            let page_start = menu_state.item_page * ITEMS_PER_PAGE;
            for i in 0..ITEMS_PER_PAGE {
                if let Some(item_name) = player_state.inventory.get(page_start + i) {
                    let col = i % 2;
                    let row = i / 2;
                    let x_offset = if col == 0 { 0.0 } else { 240.0 };
                    let y_offset = (row as f32) * 32.0;
                    commands.spawn((
                        Text2d::new(format!("* {}", item_name)),
                        menu_font.clone(),
                        menu_color,
                        Anchor::TopLeft,
                        Transform::from_translation(gml_to_bevy(start_x + x_offset, start_y + y_offset) + Vec3::new(0.0, 0.0, Z_TEXT)),
                        MenuTextItem { layer, index: i as i32 },
                        Cleanup,
                    ));
                }
            }
            
            let page_x = start_x + 240.0;
            let page_y = start_y + 64.0; 
            
            commands.spawn((
                Text2d::new(format!("   PAGE {}", menu_state.item_page + 1)),
                menu_font.clone(),
                menu_color,
                Anchor::TopLeft,
                Transform::from_translation(gml_to_bevy(page_x, page_y) + Vec3::new(0.0, 0.0, Z_TEXT)),
                MenuTextItem { layer, index: 99 },
                Cleanup,
            ));

        } else if layer == MENU_LAYER_MERCY {
            let options = ["* Spare", "* Flee"];
            for (i, opt) in options.iter().enumerate() {
                commands.spawn((
                    Text2d::new(*opt),
                    menu_font.clone(),
                    menu_color,
                    Anchor::TopLeft,
                    Transform::from_translation(gml_to_bevy(start_x, start_y + (i as f32 * 32.0)) + Vec3::new(0.0, 0.0, Z_TEXT)),
                    MenuTextItem { layer, index: i as i32 },
                    Cleanup,
                ));
            }
        }
    }
}

// バトルボックス補間
pub fn update_box_size(mut box_res: ResMut<BattleBox>, time: Res<Time>) {
    let speed = 15.0 * time.delta_secs();
    box_res.current.min.x += (box_res.target.min.x - box_res.current.min.x) * speed;
    box_res.current.min.y += (box_res.target.min.y - box_res.current.min.y) * speed;
    box_res.current.max.x += (box_res.target.max.x - box_res.current.max.x) * speed;
    box_res.current.max.y += (box_res.target.max.y - box_res.current.max.y) * speed;
}

// バトルボックス描画
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

// HP表示更新
pub fn draw_ui_status(
    player_state: Res<PlayerState>,
    mut red_bar: Query<&mut Sprite, (With<HpBarRed>, Without<HpBarYellow>)>,
    mut yel_bar: Query<&mut Sprite, (With<HpBarYellow>, Without<HpBarRed>)>,
    mut hp_text_query: Query<(&mut Text2d, &mut Transform), (With<HpText>, Without<LvText>)>,
    mut lv_text_query: Query<&mut Text2d, (With<LvText>, Without<HpText>)>,
    mut name_text_query: Query<&mut Text2d, (With<PlayerNameText>, Without<HpText>, Without<LvText>)>,
) {
    let bar_scale = 1.2; let height = 20.0;   
    
    if let Ok(mut s) = red_bar.get_single_mut() { s.custom_size = Some(Vec2::new(player_state.max_hp * bar_scale, height)); }
    if let Ok(mut s) = yel_bar.get_single_mut() { s.custom_size = Some(Vec2::new(player_state.hp * bar_scale, height)); }
    
    if let Ok((mut t, mut trans)) = hp_text_query.get_single_mut() {
        t.0 = format!("{:.0} / {:.0}", player_state.hp, player_state.max_hp);
        let visual_hp_bar_x = 250.0;
        let text_x = visual_hp_bar_x + (player_state.max_hp * bar_scale) + 15.0;
        trans.translation = gml_to_bevy(text_x, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT);
    }

    if let Ok(mut t) = lv_text_query.get_single_mut() {
        t.0 = format!("LV {}", player_state.lv);
    }

    if let Ok(mut t) = name_text_query.get_single_mut() {
        t.0 = player_state.name.clone();
    }
}

// ボタン選択表示
pub fn update_button_sprites(
    combat_state: Res<CombatState>,
    menu_state: Res<MenuState>,
    mut query: Query<(&ButtonVisual, &mut Sprite)>,
) {
    for (btn, mut sprite) in query.iter_mut() {
        if combat_state.mn_fight == MainFightState::Menu && menu_state.menu_layer == MENU_LAYER_TOP && btn.index == menu_state.menu_coords[MENU_LAYER_TOP as usize] {
            sprite.image = btn.selected_texture.clone();
        } else {
            sprite.image = btn.normal_texture.clone();
        }
    }
}

// テキスト送り
pub fn animate_text(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut combat_state: ResMut<CombatState>,
    mut menu_state: ResMut<MenuState>,
    mut query: Query<(Entity, &mut Typewriter, &mut Text2d)>,
) {
    for (entity, mut writer, mut text) in query.iter_mut() {
        if writer.finished { 
            if combat_state.my_fight == MessageFightState::PlayerActionText {
                if input.just_pressed(KeyCode::KeyZ) {
                     commands.entity(entity).despawn();
                     combat_state.my_fight = MessageFightState::None;
                     combat_state.mn_fight = MainFightState::EnemyDialog; 
                     combat_state.bubble_timer.reset(); 
                     menu_state.menu_layer = MENU_LAYER_TOP;
                }
            }
            continue; 
        }
        if input.just_pressed(KeyCode::KeyX) {
            writer.visible_chars = writer.full_text.chars().count();
            text.0 = writer.full_text.clone();
            writer.finished = true; continue;
        }
        if writer.timer.tick(time.delta()).just_finished() {
            let char_count = writer.full_text.chars().count();
            if writer.visible_chars < char_count {
                writer.visible_chars += 1;
                let displayed: String = writer.full_text.chars().take(writer.visible_chars).collect();
                text.0 = displayed;
            } else { writer.finished = true; }
        }
    }
}

// 敵頭揺れ
pub fn animate_enemy_head(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut EnemyHead)>,
) {
    for (mut transform, mut head) in query.iter_mut() {
        head.timer += time.delta_secs();
        let offset = (head.timer * 2.0).sin() * 2.0; 
        transform.translation.y = head.base_y + offset;
    }
}
