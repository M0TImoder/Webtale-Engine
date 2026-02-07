use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_egui::EguiContexts;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;
use crate::systems::phase;

// 攻撃バー
pub fn attack_bar_update(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut combat_state: ResMut<CombatState>,
    player_state: Res<PlayerState>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut Transform, &mut AttackBar, &mut Sprite)>,
    enemy_query: Query<&Transform, (With<EnemyBody>, Without<AttackBar>)>,
    mut egui_contexts: EguiContexts,
    editor_query: Query<Entity, With<EditorWindow>>,
) {
    if let Ok(editor_entity) = editor_query.get_single() {
        if egui_contexts.ctx_for_entity_mut(editor_entity).wants_keyboard_input() {
            return;
        }
    }

    if combat_state.mn_fight != MainFightState::PlayerAttackBar && combat_state.mn_fight != MainFightState::PlayerAttackResolve { return; }

    for (bar_entity, mut transform, mut bar, mut sprite) in query.iter_mut() {
        if bar.moving {
            transform.translation.x += bar.speed * time.delta_secs();
            
            if bar.flash_state {
                 bar.flash_state = false;
                 sprite.image = asset_server.load("texture/attack/spr_targetchoice_0.png");
            }

            let box_center_x = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 0.0).x;
            let miss_threshold = box_center_x + 280.0;
            let auto_press = transform.translation.x > miss_threshold;

            if input.just_pressed(KeyCode::KeyZ) || auto_press {
                if auto_press {
                    commands.entity(bar_entity).despawn();
                } else {
                    bar.moving = false;
                    sprite.image = asset_server.load("texture/attack/spr_targetchoice_1.png");
                    bar.flash_state = true; 
                }
                
                let distance = (transform.translation.x - box_center_x).abs();
                
                let base_damage = player_state.attack;
                
                let damage = if distance < 12.0 {
                    (base_damage * 2.2) as i32 
                } else {
                    let stretch = (280.0 - distance).max(0.0) / 280.0;
                    (base_damage * stretch * 2.0) as i32
                };

                let enemy_pos = if let Ok(e_trans) = enemy_query.get_single() {
                    e_trans.translation
                } else {
                    gml_to_bevy(320.0, 136.0)
                };

                let wait_time = if damage > 0 {
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite { image: asset_server.load("texture/attack/spr_strike_0.png"), ..default() },
                            transform: Transform {
                                translation: enemy_pos + Vec3::new(0.0, 0.0, Z_SLICE),
                                scale: Vec3::splat(2.0),
                                ..default()
                            },
                            ..default()
                        },
                        SliceEffect { timer: Timer::from_seconds(0.15, TimerMode::Repeating), frame_index: 0 },
                        Cleanup,
                    ));
                    0.9 
                } else {
                    0.0
                };

                commands.spawn(PendingDamage {
                    timer: Timer::from_seconds(wait_time, TimerMode::Once),
                    damage,
                    target_pos: enemy_pos,
                });

                combat_state.mn_fight = MainFightState::PlayerAttackResolve; 
            }
        } else {
            if bar.flash_timer.tick(time.delta()).just_finished() {
                bar.flash_state = !bar.flash_state;
                let path = if bar.flash_state { "texture/attack/spr_targetchoice_1.png" } else { "texture/attack/spr_targetchoice_0.png" };
                sprite.image = asset_server.load(path);
            }
        }
    }
}

// ダメージ適用
pub fn apply_pending_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_state: ResMut<EnemyState>,
    mut combat_state: ResMut<CombatState>,
    mut menu_state: ResMut<MenuState>,
    asset_server: Res<AssetServer>,
    _game_fonts: Res<GameFonts>,
    python_runtime: NonSend<PythonRuntime>,
    mut query: Query<(Entity, &mut PendingDamage)>,
) {
    for (entity, mut pending) in query.iter_mut() {
        if pending.timer.tick(time.delta()).finished() {
            let old_hp = enemy_state.hp;
            enemy_state.hp = (enemy_state.hp - pending.damage).max(0);
            let damage = pending.damage;
            let enemy_pos = pending.target_pos;
            combat_state.last_act_command = None;
            combat_state.last_player_action = if damage > 0 {
                "attackHit".to_string()
            } else {
                "attackMiss".to_string()
            };
            if let Some(next_phase) = phase::apply_phase_update(&mut enemy_state, &mut combat_state, &mut menu_state, PROJECT_NAME, "damage", &python_runtime) {
                if next_phase != combat_state.phase_name {
                    combat_state.phase_name = next_phase;
                    combat_state.phase_turn = 0;
                }
            }

            let text_start_pos = enemy_pos + Vec3::new(0.0, 50.0, Z_DAMAGE_TEXT);

            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(text_start_pos),
                    ..default()
                },
                DamageNumber { 
                    timer: Timer::from_seconds(1.2, TimerMode::Once),
                    velocity_y: 240.0, 
                    gravity: 900.0,
                    start_y: text_start_pos.y,
                },
                Cleanup,
            )).with_children(|parent| {
                let scale = 1.25;

                if damage > 0 {
                    let dmg_str = format!("{}", damage);
                    let char_spacing = 42.0; 

                    let total_width = (dmg_str.chars().count() as f32) * char_spacing;
                    let start_xoffset = -(total_width / 2.0) + (char_spacing / 2.0);

                    for (i, char) in dmg_str.chars().enumerate() {
                        let char_x = start_xoffset + (i as f32 * char_spacing);
                        let texture_path = format!("texture/dmgnum/spr_dmgnum_o_{}.png", char);

                        parent.spawn(SpriteBundle {
                            sprite: Sprite { 
                                image: asset_server.load(texture_path),
                                color: Color::rgb(0.8, 0.0, 0.0), 
                                custom_size: None,
                                ..default() 
                            },
                            transform: Transform::from_xyz(char_x, 0.0, 0.0).with_scale(Vec3::splat(scale)),
                            ..default()
                        });
                    }
                } else {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite {
                            image: asset_server.load("texture/dmgnum/spr_dmgmiss_o.png"),
                            color: Color::rgb(0.8, 0.8, 0.8), 
                            custom_size: None,
                            ..default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(scale)),
                        ..default()
                    });
                }
            });

            if damage > 0 {
                let bar_width_max = 140.0;
                let bar_height = 14.0;
                let bar_pos = enemy_pos + Vec3::new(0.0, 20.0, Z_DAMAGE_HP_BAR);

                commands.spawn((
                    SpatialBundle {
                        transform: Transform::from_translation(bar_pos),
                        ..default()
                    },
                    EnemyHpBar {
                        lifespan: Timer::from_seconds(1.2, TimerMode::Once),
                        animation: Timer::from_seconds(1.0, TimerMode::Once),
                        start_width: (old_hp as f32 / enemy_state.max_hp as f32) * bar_width_max,
                        target_width: (enemy_state.hp as f32 / enemy_state.max_hp as f32) * bar_width_max,
                    },
                    Cleanup,
                )).with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite { color: Color::srgb(0.25, 0.25, 0.25), custom_size: Some(Vec2::new(bar_width_max, bar_height)), ..default() },
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)), 
                        ..default()
                    });
                    let left_offset = -bar_width_max / 2.0;
                    parent.spawn((
                        SpriteBundle {
                            sprite: Sprite { 
                                color: Color::rgb(0.0, 1.0, 0.0), 
                                custom_size: Some(Vec2::new((old_hp as f32 / enemy_state.max_hp as f32) * bar_width_max, bar_height)),
                                anchor: Anchor::CenterLeft, 
                                ..default() 
                            },
                            transform: Transform::from_translation(Vec3::new(left_offset, 0.0, 0.0)),
                            ..default()
                        },
                        EnemyHpBarForeground,
                    ));
                });
            }

            commands.entity(entity).despawn();
        }
    }
}

// 斬撃アニメ
pub fn animate_slice_effect(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut SliceEffect, &mut Sprite)>,
) {
    for (entity, mut effect, mut sprite) in query.iter_mut() {
        if effect.timer.tick(time.delta()).just_finished() {
            effect.frame_index += 1;
            if effect.frame_index > 5 {
                commands.entity(entity).despawn();
            } else {
                let path = format!("texture/attack/spr_strike_{}.png", effect.frame_index);
                sprite.image = asset_server.load(path);
            }
        }
    }
}

// ダメージ表示
pub fn damage_number_update(
    mut commands: Commands,
    time: Res<Time>,
    enemy_state: Res<EnemyState>,
    mut combat_state: ResMut<CombatState>,
    mut menu_state: ResMut<MenuState>,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber), Without<EnemyBody>>,
    attack_bar_query: Query<Entity, With<AttackBar>>,
    target_box_query: Query<Entity, With<AttackTargetBox>>,
    mut enemy_query: Query<(Entity, &mut Sprite, &Transform), With<EnemyBody>>,
) {
    for (entity, mut transform, mut dmg) in query.iter_mut() {
        dmg.timer.tick(time.delta());
        
        transform.translation.y += dmg.velocity_y * time.delta_secs();
        dmg.velocity_y -= dmg.gravity * time.delta_secs();

        if transform.translation.y < dmg.start_y {
            transform.translation.y = dmg.start_y;
            dmg.velocity_y = 0.0;
            dmg.gravity = 0.0;
        }

        if dmg.timer.finished() {
            commands.entity(entity).despawn_recursive();
            
            for bar_entity in attack_bar_query.iter() { commands.entity(bar_entity).despawn(); }
            for box_entity in target_box_query.iter() { commands.entity(box_entity).despawn(); }
            
            if enemy_state.hp <= 0 {
                for (e_entity, sprite, e_transform) in enemy_query.iter_mut() {
                    commands.entity(e_entity).insert(Vaporizing {
                        scan_line: 0.0,
                        image_handle: sprite.image.clone(),
                        initial_y: e_transform.translation.y,
                    });
                }
                combat_state.mn_fight = MainFightState::Menu; 
            } else {
                combat_state.mn_fight = MainFightState::EnemyDialog; 
                combat_state.bubble_timer.reset(); 
                menu_state.menu_layer = MENU_LAYER_TOP;
            }
        }
    }
}

// 敵HPバー
pub fn enemy_hp_bar_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EnemyHpBar, &Children)>,
    mut bar_sprite_query: Query<&mut Sprite, With<EnemyHpBarForeground>>,
) {
    for (entity, mut bar, children) in query.iter_mut() {
        bar.lifespan.tick(time.delta());
        bar.animation.tick(time.delta());

        let t = bar.animation.fraction();
        let current_width = bar.start_width + (bar.target_width - bar.start_width) * t;

        for &child in children.iter() {
            if let Ok(mut sprite) = bar_sprite_query.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(current_width, 14.0));
            }
        }

        if bar.lifespan.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
