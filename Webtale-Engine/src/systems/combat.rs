use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;
use std::f32::consts::PI;
use bevy_egui::EguiContexts;
use pyo3::prelude::*;
use pyo3::types::PyDict; 
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn battle_flow_control(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    game_fonts: Res<GameFonts>,
    _time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>, 
    mut box_res: ResMut<BattleBox>,
    bubbles: Query<Entity, With<SpeechBubble>>,
    bubble_text_query: Query<&Typewriter, With<SpeechBubble>>, 
    mut soul_query: Query<&mut Transform, With<Soul>>,
    mut egui_contexts: EguiContexts,
    editor_query: Query<Entity, With<EditorWindow>>,
) {
    if let Ok(editor_entity) = editor_query.get_single() {
        if egui_contexts.ctx_for_window_mut(editor_entity).wants_keyboard_input() {
            return;
        }
    }

    if game_state.mnfight == 1 {
        if bubbles.is_empty() {
            box_res.target = Rect::new(32.0, 250.0, 602.0, 385.0);
            let bubble_x = 320.0 + 40.0; 
            let bubble_y = 160.0 - 95.0; 
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("blcon/spr_blconsm.png"), 
                    sprite: Sprite { 
                        color: Color::WHITE, 
                        custom_size: Some(Vec2::new(100.0, 80.0)), 
                        anchor: Anchor::TopLeft, 
                        ..default() 
                    },
                    transform: Transform::from_translation(gml_to_bevy(bubble_x, bubble_y) + Vec3::new(0.0, 0.0, Z_BUBBLE)),
                    ..default()
                },
                SpeechBubble,
                Cleanup,
            ));
            let messages = ["Ribbit, ribbit.", "Croak.", "Hop, hop."];
            let msg = messages[rand::thread_rng().gen_range(0..messages.len())];
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("", TextStyle { font: game_fonts.dialog.clone(), font_size: 24.0, color: Color::BLACK }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(bubble_x + 15.0, bubble_y + 15.0) + Vec3::new(0.0, 0.0, Z_BUBBLE_TEXT)),
                    ..default()
                },
                Typewriter { full_text: msg.to_string(), visible_chars: 0, timer: Timer::from_seconds(0.05, TimerMode::Repeating), finished: false },
                SpeechBubble, 
                Cleanup,
            ));
        }
        
        let mut is_finished = false;
        if let Ok(writer) = bubble_text_query.get_single() {
            if writer.finished {
                is_finished = true;
            }
        }

        if is_finished && input.just_pressed(KeyCode::KeyZ) {
            for entity in bubbles.iter() { commands.entity(entity).despawn_recursive(); }
            
            game_state.mnfight = 2; 
            game_state.turntimer = -1.0; 
            
            box_res.target = Rect::new(217.0, 125.0, 417.0, 385.0);
            
            let box_center_x = (217.0 + 417.0) / 2.0;
            let box_center_y = (125.0 + 385.0) / 2.0;
            if let Ok(mut t) = soul_query.get_single_mut() {
                t.translation = gml_to_bevy(box_center_x, box_center_y) + Vec3::new(0.0, 0.0, Z_SOUL);
            }
        }
    }
}

pub fn combat_turn_manager(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut battle_box: ResMut<BattleBox>,
    bullet_query: Query<Entity, With<PythonBullet>>,
    mut scripts: ResMut<DanmakuScripts>,
) {
    if game_state.mnfight == 2 {
        if game_state.turntimer < 0.0 {
            game_state.turntimer = 5.0;
            
            // Load and run python script init
            let script_content = std::fs::read_to_string("assets/danmaku/frog_jump.py").unwrap_or_default();
            
            Python::with_gil(|py| {
                let sys = PyModule::import_bound(py, "sys").expect("Failed to import sys");
                let path = sys.getattr("path").expect("Failed to get sys.path");
                let env_path = std::env::current_dir().unwrap().join("assets").join("danmaku");
                let _ = path.call_method1("append", (env_path.to_str().unwrap(),));

                let module = PyModule::from_code_bound(py, &script_content, "frog_jump.py", "frog_jump").expect("Failed to load python script");
                
                // init()
                let init_func = module.getattr("init").expect("Failed to get init function");
                let init_result = init_func.call0().expect("Failed to call init");
                let init_data: &Bound<PyDict> = init_result.downcast().expect("init should return dict");
                
                let box_data_obj = init_data.get_item("box").expect("box missing").expect("box None");
                let _box_data: Vec<f32> = box_data_obj.extract().expect("box format error");
                // Update battle box
                // box_res.target = Rect::new(box_data[0], box_data[1], box_data[2], box_data[3]); 
                // Setting initial box target in `battle_flow_control` is enough for now, 
                // or we can update it here if needed dynamically.

                let texture_path_obj = init_data.get_item("texture_wait").expect("texture_wait missing").expect("texture_wait None");
                let texture_path: String = texture_path_obj.extract().unwrap();
                
                let spawn_x = ORIGIN_X + battle_box.current.max.x - 40.0;
                let spawn_y = ORIGIN_Y - battle_box.current.max.y + 40.0;

                // spawn()
                let spawn_func = module.getattr("spawn").expect("Failed to get spawn function");
                let bullet_obj: PyObject = spawn_func.call0().expect("Failed to call spawn").into();
                
                // Initialize bullet with position
                let bullet_bound = bullet_obj.bind(py);
                let _ = bullet_bound.call_method1("set_pos", (spawn_x, spawn_y));

                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load(texture_path),
                        transform: Transform::from_xyz(spawn_x, spawn_y, 30.0).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    PythonBullet {
                        script_name: "frog_jump".to_string(),
                        bullet_data: bullet_obj,
                        damage: 4,
                    },
                    Cleanup,
                ));
                
                scripts.modules.insert("frog_jump".to_string(), module.into());
            });
        }

        game_state.turntimer -= time.delta_seconds();

        if game_state.turntimer <= 0.0 {
            for entity in bullet_query.iter() {
                commands.entity(entity).despawn();
            }
            
            game_state.mnfight = 3;
            game_state.turntimer = -1.0;
        }
    } else if game_state.mnfight == 3 {
        game_state.mnfight = 0;
        game_state.myfight = 0;
        game_state.menu_layer = 0;
        game_state.dialog_text = "* Froggit hops close!".to_string(); 
        
        battle_box.target = Rect::new(32.0, 250.0, 602.0, 385.0);
    }
}

pub fn leapfrog_bullet_update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut Transform, &PythonBullet, &mut Handle<Image>)>,
    _scripts: Res<DanmakuScripts>,
) {
    let dt = time.delta_seconds();
    
    Python::with_gil(|py| {
        for (entity, mut transform, bullet, mut texture) in query.iter_mut() {
            // We don't really need the module object anymore to call methods on the instance,
            // but we keep the structure. Actually, we can just use the object directly.
            
            let bullet_obj = bullet.bullet_data.bind(py);
            
            // Call sys_update(dt)
            if let Err(e) = bullet_obj.call_method1("sys_update", (dt,)) {
                e.print(py);
                continue;
            }
            
            // Retrieve properties to sync with Rust
            
            // Sync Position
            if let Ok(x) = bullet_obj.getattr("x").and_then(|v| v.extract::<f32>()) {
                if let Ok(y) = bullet_obj.getattr("y").and_then(|v| v.extract::<f32>()) {
                     transform.translation.x = x;
                     transform.translation.y = y;
                }
            }

            // Sync Texture if changed
            if let Ok(texture_val) = bullet_obj.getattr("texture") {
                 if !texture_val.is_none() {
                      if let Ok(path) = texture_val.extract::<String>() {
                           *texture = asset_server.load(path);
                           // Reset texture field to None to avoid reloading every frame (optional optimization)
                           // For now we leave it or rely on setting it to None in python if we wanted one-shot,
                           // but here we just load what is there.
                           // Actually, to prevent repeated loading, checking equality or having a "changed" flag is better.
                           // But for simplicity, we let Bevy asset server handle caching.
                      }
                 }
            }
            
            // Check deletion
            if let Ok(should_delete) = bullet_obj.getattr("should_delete").and_then(|v| v.extract::<bool>()) {
                if should_delete {
                    commands.entity(entity).despawn();
                }
            }
        }
    });
}

pub fn attack_bar_update(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut Transform, &mut AttackBar, &mut Handle<Image>)>,
    enemy_query: Query<&Transform, (With<EnemyBody>, Without<AttackBar>)>,
    mut egui_contexts: EguiContexts,
    editor_query: Query<Entity, With<EditorWindow>>,
) {
    if let Ok(editor_entity) = editor_query.get_single() {
        if egui_contexts.ctx_for_window_mut(editor_entity).wants_keyboard_input() {
            return;
        }
    }

    if game_state.mnfight != 4 && game_state.mnfight != 5 { return; }

    for (bar_entity, mut transform, mut bar, mut texture) in query.iter_mut() {
        if bar.moving {
            transform.translation.x += bar.speed * time.delta_seconds();
            
            if bar.flash_state {
                 bar.flash_state = false;
                 *texture = asset_server.load("attack/spr_targetchoice_0.png");
            }

            let box_center_x = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 0.0).x;
            let miss_threshold = box_center_x + 280.0;
            let auto_press = transform.translation.x > miss_threshold;

            if input.just_pressed(KeyCode::KeyZ) || auto_press {
                if auto_press {
                    commands.entity(bar_entity).despawn();
                } else {
                    bar.moving = false;
                    *texture = asset_server.load("attack/spr_targetchoice_1.png");
                    bar.flash_state = true; 
                }
                
                let distance = (transform.translation.x - box_center_x).abs();
                
                let base_damage = game_state.attack;
                
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
                            texture: asset_server.load("attack/spr_strike_0.png"),
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

                game_state.mnfight = 5; 
            }
        } else {
            if bar.flash_timer.tick(time.delta()).just_finished() {
                bar.flash_state = !bar.flash_state;
                let path = if bar.flash_state { "attack/spr_targetchoice_1.png" } else { "attack/spr_targetchoice_0.png" };
                *texture = asset_server.load(path);
            }
        }
    }
}

pub fn apply_pending_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    _game_fonts: Res<GameFonts>,
    mut query: Query<(Entity, &mut PendingDamage)>,
) {
    for (entity, mut pending) in query.iter_mut() {
        if pending.timer.tick(time.delta()).finished() {
            let old_hp = game_state.enemy_hp;
            game_state.enemy_hp = (game_state.enemy_hp - pending.damage).max(0);
            let damage = pending.damage;
            let enemy_pos = pending.target_pos;

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
                    let start_x_offset = -(total_width / 2.0) + (char_spacing / 2.0);

                    for (i, char) in dmg_str.chars().enumerate() {
                        let char_x = start_x_offset + (i as f32 * char_spacing);
                        let texture_path = format!("dmgnum/spr_dmgnum_o_{}.png", char);

                        parent.spawn(SpriteBundle {
                            texture: asset_server.load(texture_path),
                            sprite: Sprite { 
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
                        texture: asset_server.load("dmgnum/spr_dmgmiss_o.png"),
                        sprite: Sprite {
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
                        start_width: (old_hp as f32 / game_state.enemy_max_hp as f32) * bar_width_max,
                        target_width: (game_state.enemy_hp as f32 / game_state.enemy_max_hp as f32) * bar_width_max,
                    },
                    Cleanup,
                )).with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite { color: Color::DARK_GRAY, custom_size: Some(Vec2::new(bar_width_max, bar_height)), ..default() },
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)), 
                        ..default()
                    });
                    let left_offset = -bar_width_max / 2.0;
                    parent.spawn((
                        SpriteBundle {
                            sprite: Sprite { 
                                color: Color::rgb(0.0, 1.0, 0.0), 
                                custom_size: Some(Vec2::new((old_hp as f32 / game_state.enemy_max_hp as f32) * bar_width_max, bar_height)),
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

pub fn animate_slice_effect(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut SliceEffect, &mut Handle<Image>)>,
) {
    for (entity, mut effect, mut texture) in query.iter_mut() {
        if effect.timer.tick(time.delta()).just_finished() {
            effect.frame_index += 1;
            if effect.frame_index > 5 {
                commands.entity(entity).despawn();
            } else {
                let path = format!("attack/spr_strike_{}.png", effect.frame_index);
                *texture = asset_server.load(path);
            }
        }
    }
}

pub fn damage_number_update(
    mut commands: Commands,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber), Without<EnemyBody>>,
    attack_bar_query: Query<Entity, With<AttackBar>>,
    target_box_query: Query<Entity, With<AttackTargetBox>>,
    mut enemy_query: Query<(Entity, &mut Sprite, &Transform, &Handle<Image>), With<EnemyBody>>,
) {
    for (entity, mut transform, mut dmg) in query.iter_mut() {
        dmg.timer.tick(time.delta());
        
        transform.translation.y += dmg.velocity_y * time.delta_seconds();
        dmg.velocity_y -= dmg.gravity * time.delta_seconds();

        if transform.translation.y < dmg.start_y {
            transform.translation.y = dmg.start_y;
            dmg.velocity_y = 0.0;
            dmg.gravity = 0.0;
        }

        if dmg.timer.finished() {
            commands.entity(entity).despawn_recursive();
            
            for bar_entity in attack_bar_query.iter() { commands.entity(bar_entity).despawn(); }
            for box_entity in target_box_query.iter() { commands.entity(box_entity).despawn(); }
            
            if game_state.enemy_hp <= 0 {
                for (e_entity, _, e_transform, handle) in enemy_query.iter_mut() {
                    commands.entity(e_entity).insert(Vaporizing {
                        scan_line: 0.0,
                        image_handle: handle.clone(),
                        initial_y: e_transform.translation.y,
                    });
                }
                game_state.mnfight = 0; 
            } else {
                game_state.mnfight = 1; 
                game_state.bubble_timer.reset(); 
                game_state.menu_layer = MENU_LAYER_TOP;
            }
        }
    }
}

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

pub fn vaporize_enemy_system(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Image>>,
    mut query: Query<(Entity, &mut Vaporizing, &mut Sprite, &mut Transform)>,
) {
    let scan_speed = 100.0; 
    let pixel_size = 2.0;

    for (entity, mut vap, mut sprite, mut transform) in query.iter_mut() {
        let image = if let Some(img) = assets.get(&vap.image_handle) {
            img
        } else {
            continue;
        };

        let texture_width = image.texture_descriptor.size.width as f32;
        let texture_height = image.texture_descriptor.size.height as f32;
        
        let prev_line = vap.scan_line;
        vap.scan_line += scan_speed * time.delta_seconds();

        let start_y = prev_line as u32;
        let end_y = (vap.scan_line as u32).min(texture_height as u32);
        
        let step = if pixel_size < 1.0 { 1 } else { pixel_size as u32 };

        for y in (start_y..end_y).step_by(step as usize) {
            for x in (0..(texture_width as u32)).step_by(step as usize) {
                let index = ((y * (texture_width as u32) + x) * 4) as usize;
                if index + 3 >= image.data.len() { continue; }

                let alpha = image.data[index + 3];

                if alpha > 10 { 
                    let sprite_scale = transform.scale.x; 
                    
                    let relative_x = (x as f32 - texture_width / 2.0) * sprite_scale;
                    let relative_y = (texture_height / 2.0 - y as f32) * sprite_scale;

                    let dust_pos = Vec3::new(
                        transform.translation.x + relative_x,
                        vap.initial_y + relative_y, 
                        0.1
                    );

                    let mut rng = rand::thread_rng();
                    let velocity_x = rng.gen_range(-80.0..80.0);
                    let velocity_y = rng.gen_range(20.0..80.0);
                    let max_alpha = rng.gen_range(0.2..1.0);

                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite { 
                                color: Color::rgba(1.0, 1.0, 1.0, max_alpha), 
                                custom_size: Some(Vec2::splat(pixel_size * sprite_scale)), 
                                ..default() 
                            },
                            transform: Transform::from_translation(dust_pos),
                            ..default()
                        },
                        DustParticle {
                            velocity: Vec3::new(velocity_x, velocity_y, 0.0), 
                            timer: Timer::from_seconds(1.0, TimerMode::Once),
                            max_alpha,
                        },
                        Cleanup,
                    ));
                }
            }
        }

        let current_height_px = (texture_height - vap.scan_line).max(0.0);
        
        if current_height_px <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            sprite.rect = Some(Rect {
                min: Vec2::new(0.0, vap.scan_line),
                max: Vec2::new(texture_width, texture_height),
            });
            
            let scale = transform.scale.y;
            sprite.custom_size = Some(Vec2::new(texture_width * scale, current_height_px * scale));

            let removed_height = texture_height - current_height_px;
            transform.translation.y = vap.initial_y - (removed_height * scale / 2.0);
        }
    }
}

pub fn dust_particle_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut DustParticle)>,
) {
    for (entity, mut transform, mut sprite, mut dust) in query.iter_mut() {
        dust.timer.tick(time.delta());
        
        transform.translation += dust.velocity * time.delta_seconds();

        let alpha = dust.max_alpha * (1.0 - dust.timer.fraction());
        sprite.color.set_a(alpha);

        if dust.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn soul_collision_detection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    mut soul_query: Query<(Entity, &Transform), With<Soul>>,
    bullet_query: Query<(&Transform, &PythonBullet)>,
    mut visibility_param_set: ParamSet<(
        Query<&mut Visibility, (With<Sprite>, Without<Soul>, Without<EditorWindow>)>,
        Query<&mut Visibility, (With<Text>, Without<Soul>, Without<EditorWindow>)>,
    )>,
) {
    if game_state.invincibility_timer > 0.0 {
        return;
    }

    if let Ok((soul_entity, soul_tf)) = soul_query.get_single_mut() {
        let soul_radius = 6.0;
        let bullet_radius = 10.0;

        for (bullet_tf, bullet) in bullet_query.iter() {
            let distance = soul_tf.translation.distance(bullet_tf.translation);
            if distance < (soul_radius + bullet_radius) {
                game_state.hp -= bullet.damage as f32;
                
                game_state.invincibility_timer = game_state.invincibility_duration;

                if game_state.hp <= 0.0 { 
                    game_state.hp = 0.0; 
                    game_state.mnfight = 99;

                    for mut visibility in visibility_param_set.p0().iter_mut() {
                        *visibility = Visibility::Hidden;
                    }
                    for mut visibility in visibility_param_set.p1().iter_mut() {
                        *visibility = Visibility::Hidden;
                    }

                    commands.entity(soul_entity).despawn();

                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(Vec2::new(10000.0, 10000.0)), 
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0)),
                            ..default()
                        },
                        Cleanup,
                    ));

                    commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load("heart/spr_heart_0.png"), 
                            sprite: Sprite { 
                                color: Color::WHITE, 
                                custom_size: Some(Vec2::new(16.0, 16.0)), 
                                ..default() 
                            },
                            transform: Transform::from_translation(Vec3::new(soul_tf.translation.x, soul_tf.translation.y, 600.0)),
                            ..default()
                        },
                        HeartDefeated {
                            timer: Timer::from_seconds(1.0, TimerMode::Once), 
                            state: HeartDefeatedState::InitialDelay,
                            original_pos: soul_tf.translation,
                        },
                        Cleanup,
                    ));
                    
                    break;
                }
            }
        }
    }
}
pub fn invincibility_update(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut soul_query: Query<&mut Visibility, With<Soul>>,
) {
    if game_state.invincibility_timer > 0.0 {
        game_state.invincibility_timer -= time.delta_seconds();

        if let Ok(mut visibility) = soul_query.get_single_mut() {
            if game_state.invincibility_timer <= 0.0 {
                game_state.invincibility_timer = 0.0;
                *visibility = Visibility::Inherited;
            } else {
                let blink_interval = 1.0 / 15.0; 
                let blink_state = (game_state.invincibility_timer / blink_interval).ceil() as i32;
                
                if blink_state % 2 == 0 {
                    *visibility = Visibility::Hidden;
                } else {
                    *visibility = Visibility::Inherited;
                }
            }
        }
    }
}

pub fn heart_defeated_update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut HeartDefeated, &mut Transform, &mut Handle<Image>)>,
) {
    for (entity, mut defeated, mut transform, mut texture) in query.iter_mut() {
        defeated.timer.tick(time.delta());

        match defeated.state {
            HeartDefeatedState::InitialDelay => {
                if defeated.timer.finished() {
                    defeated.state = HeartDefeatedState::Cracked;
                    defeated.timer = Timer::from_seconds(1.0, TimerMode::Once); 
                    
                    *texture = asset_server.load("heart/spr_heartbreak.png");
                    transform.translation.x -= 2.0; 
                }
            },
            HeartDefeatedState::Cracked => {
                if defeated.timer.finished() {
                    let base_pos = transform.translation;
                    let offsets = [
                        Vec3::new(-2.0, 0.0, 0.0),
                        Vec3::new(0.0, -3.0, 0.0),
                        Vec3::new(2.0, -6.0, 0.0),
                        Vec3::new(8.0, 0.0, 0.0),
                        Vec3::new(10.0, -3.0, 0.0),
                        Vec3::new(12.0, -6.0, 0.0),
                    ];

                    for offset in offsets.iter() {
                        let mut rng = rand::thread_rng();
                        let direction_deg = rng.gen_range(0.0..360.0);
                        let direction_rad = direction_deg * PI / 180.0;
                        let speed = 7.0 * 30.0;
                        
                        let vx = speed * direction_rad.cos();
                        let vy = speed * direction_rad.sin(); 

                        let shard_index = rng.gen_range(0..4);
                        let texture_path = format!("heart/spr_heartshards_{}.png", shard_index);

                        commands.spawn((
                            SpriteBundle {
                                texture: asset_server.load(texture_path),
                                transform: Transform::from_translation(base_pos + *offset + Vec3::new(0.0, 0.0, 0.0)).with_translation(Vec3::new(base_pos.x + offset.x, base_pos.y + offset.y, 600.0)), 
                                ..default()
                            },
                            HeartShard {
                                velocity: Vec3::new(vx, vy, 0.0),
                                gravity: 0.2 * 30.0 * 30.0, 
                            },
                            Cleanup,
                        ));
                    }

                    commands.spawn((
                        GameOverSequence {
                            timer: Timer::from_seconds(1.0, TimerMode::Once),
                            state: GameOverSequenceState::Delay,
                        },
                        Cleanup,
                    ));

                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn game_over_sequence_update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut GameOverSequence>,
    mut logo_query: Query<&mut Sprite, With<GameOverLogo>>,
) {
    for mut sequence in query.iter_mut() {
        sequence.timer.tick(time.delta());

        match sequence.state {
            GameOverSequenceState::Delay => {
                if sequence.timer.finished() {
                    sequence.state = GameOverSequenceState::FadeIn;
                    sequence.timer = Timer::from_seconds(1.0, TimerMode::Once);

                    commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load("background/spr_gameoverbg.png"),
                            sprite: Sprite {
                                color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 100.0, 700.0), 
                            ..default()
                        },
                        GameOverLogo,
                        Cleanup,
                    ));
                }
            },
            GameOverSequenceState::FadeIn => {
                let alpha = sequence.timer.fraction();
                for mut sprite in logo_query.iter_mut() {
                    sprite.color.set_a(alpha);
                }

                if sequence.timer.finished() {
                    sequence.state = GameOverSequenceState::Finished;
                    for mut sprite in logo_query.iter_mut() {
                        sprite.color.set_a(1.0);
                    }
                }
            },
            GameOverSequenceState::Finished => {
                
            }
        }
    }
}

pub fn heart_shard_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut HeartShard)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut transform, mut shard) in query.iter_mut() {
        shard.velocity.y -= shard.gravity * dt;
        transform.translation += shard.velocity * dt;

        if transform.translation.y < -300.0 {
            commands.entity(entity).despawn();
        }
    }
}
