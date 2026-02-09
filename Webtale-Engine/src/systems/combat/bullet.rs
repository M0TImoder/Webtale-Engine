use bevy::prelude::*;
use evalexpr::{Context, ContextWithMutableVariables, Value};
use crate::components::*;
use crate::resources::*;

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Int(val) => Some(*val as f64),
        Value::Float(val) => Some(*val),
        Value::Boolean(val) => Some(if *val { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn value_to_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Boolean(val) => Some(*val),
        Value::Int(val) => Some(*val != 0),
        Value::Float(val) => Some(*val != 0.0),
        _ => None,
    }
}

// 弾幕更新
pub fn leapfrog_bullet_update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    python_runtime: NonSend<PythonRuntime>,
    mut python_query: Query<(Entity, &mut Transform, &PythonBullet, &mut Sprite), (Without<ExpressionBullet>, Without<LeapFrogBullet>)>,
    mut rust_query: Query<(&mut Transform, &mut LeapFrogBullet, &mut Sprite), (Without<ExpressionBullet>, Without<PythonBullet>)>,
    mut expr_query: Query<(Entity, &mut Transform, &mut ExpressionBullet, &mut Sprite), (Without<LeapFrogBullet>, Without<PythonBullet>)>,
    _scripts: Res<DanmakuScripts>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut bullet, mut sprite) in expr_query.iter_mut() {
        let _ = bullet.context.set_value("dt".to_string(), Value::Float(dt as f64));
        let next_t = match bullet.context.get_value("t") {
            Some(value) => value_to_f64(&value).unwrap_or(0.0) + dt as f64,
            None => dt as f64,
        };
        let _ = bullet.context.set_value("t".to_string(), Value::Float(next_t));

        let update_exprs = bullet.update_exprs.clone();
        for assignment in update_exprs.iter() {
            match assignment.expr.eval_with_context(&bullet.context) {
                Ok(value) => {
                    let _ = bullet.context.set_value(assignment.target.clone(), value);
                }
                Err(err) => {
                    println!("Warning: bullet expr {} {:?}", assignment.target, err);
                }
            }
        }

        if let Some(value) = bullet.context.get_value("x") {
            if let Some(x) = value_to_f64(&value) {
                transform.translation.x = x as f32;
            }
        }
        if let Some(value) = bullet.context.get_value("y") {
            if let Some(y) = value_to_f64(&value) {
                transform.translation.y = y as f32;
            }
        }

        let mut next_texture: Option<String> = None;
        if let Some(expr) = &bullet.texture_expr {
            if let Ok(value) = expr.eval_with_context(&bullet.context) {
                if let Value::String(text) = value {
                    next_texture = Some(text);
                }
            }
        } else if let Some(Value::String(text)) = bullet.context.get_value("texture") {
            next_texture = Some(text.clone());
        }
        if let Some(texture) = next_texture {
            if bullet.last_texture.as_deref() != Some(texture.as_str()) {
                sprite.image = asset_server.load(&texture);
                bullet.last_texture = Some(texture);
            }
        }

        let mut should_delete = false;
        if let Some(expr) = &bullet.delete_expr {
            if let Ok(value) = expr.eval_with_context(&bullet.context) {
                if let Some(result) = value_to_bool(&value) {
                    should_delete = result;
                }
            }
        } else if let Some(value) = bullet.context.get_value("delete") {
            if let Some(result) = value_to_bool(&value) {
                should_delete = result;
            }
        }
        if should_delete {
            commands.entity(entity).despawn();
        }
    }

    for (mut transform, mut bullet, mut sprite) in rust_query.iter_mut() {
        match bullet.state {
            LeapFrogState::Waiting => {
                if bullet.timer.tick(time.delta()).just_finished() {
                    bullet.state = LeapFrogState::Jumping;
                    let rad = bullet.jump_angle.to_radians();
                    bullet.velocity.x = bullet.jump_speed * rad.cos();
                    bullet.velocity.y = bullet.jump_speed * rad.sin();
                    if !bullet.jump_texture.is_empty() {
                        sprite.image = asset_server.load(&bullet.jump_texture);
                    }
                }
            }
            LeapFrogState::Jumping => {
                let gravity = bullet.gravity;
                bullet.velocity += gravity * dt;
            }
        }
        transform.translation += bullet.velocity * dt;
    }

    let has_python = python_query.iter().next().is_some();
    if has_python {
        python_runtime.interpreter.enter(|vm| {
            for (entity, mut transform, bullet, mut sprite) in python_query.iter_mut() {
                let bullet_obj = bullet.bullet_data.clone();

                let sys_update = match bullet_obj.get_attr("sysUpdate", vm) {
                    Ok(func) => func,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        continue;
                    }
                };
                if let Err(err) = vm.invoke(&sys_update, (dt,)) {
                    vm.print_exception(err.clone());
                    continue;
                }

                let x = match bullet_obj.get_attr("x", vm) {
                    Ok(value) => match value.try_into_value::<f32>(vm) {
                        Ok(result) => Some(result),
                        Err(err) => {
                            vm.print_exception(err.clone());
                            None
                        }
                    },
                    Err(err) => {
                        vm.print_exception(err.clone());
                        None
                    }
                };
                let y = match bullet_obj.get_attr("y", vm) {
                    Ok(value) => match value.try_into_value::<f32>(vm) {
                        Ok(result) => Some(result),
                        Err(err) => {
                            vm.print_exception(err.clone());
                            None
                        }
                    },
                    Err(err) => {
                        vm.print_exception(err.clone());
                        None
                    }
                };
                if let (Some(x), Some(y)) = (x, y) {
                    transform.translation.x = x;
                    transform.translation.y = y;
                }

                match bullet_obj.get_attr("texture", vm) {
                    Ok(texture_val) => match texture_val.try_into_value::<Option<String>>(vm) {
                        Ok(path) => {
                            if let Some(path) = path {
                                sprite.image = asset_server.load(path);
                            }
                        }
                        Err(err) => {
                            vm.print_exception(err.clone());
                        }
                    },
                    Err(err) => {
                        vm.print_exception(err.clone());
                    }
                }

                match bullet_obj.get_attr("shouldDelete", vm) {
                    Ok(value) => match value.try_into_value::<bool>(vm) {
                        Ok(should_delete) => {
                            if should_delete {
                                commands.entity(entity).despawn();
                            }
                        }
                        Err(err) => {
                            vm.print_exception(err.clone());
                        }
                    },
                    Err(err) => {
                        vm.print_exception(err.clone());
                    }
                }
            }
        });
    }
}

// 被弾判定
pub fn soul_collision_detection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_state: ResMut<PlayerState>,
    mut combat_state: ResMut<CombatState>,
    mut soul_query: Query<(Entity, &Transform), With<Soul>>,
    python_bullet_query: Query<(&Transform, &PythonBullet)>,
    leapfrog_bullet_query: Query<(&Transform, &LeapFrogBullet)>,
    expr_bullet_query: Query<(&Transform, &ExpressionBullet)>,
    mut visibility_param_set: ParamSet<(
        Query<&mut Visibility, (With<Sprite>, Without<Soul>, Without<EditorWindow>)>,
        Query<&mut Visibility, (With<Text2d>, Without<Soul>, Without<EditorWindow>)>,
    )>,
) {
    if player_state.invincibility_timer > 0.0 {
        return;
    }

    if let Ok((soul_entity, soul_tf)) = soul_query.get_single_mut() {
        let soul_radius = 6.0;
        let bullet_radius = 10.0;

        for (bullet_tf, bullet) in python_bullet_query.iter() {
            let distance = soul_tf.translation.distance(bullet_tf.translation);
            if distance < (soul_radius + bullet_radius) {
                player_state.hp -= bullet.damage as f32;
                player_state.invincibility_timer = player_state.invincibility_duration;
                if player_state.hp <= 0.0 { 
                    player_state.hp = 0.0; 
                    combat_state.mn_fight = MainFightState::PlayerDefeated;
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
                            sprite: Sprite { 
                                image: asset_server.load("texture/heart/spr_heart_0.png"), 
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
                    return;
                }
            }
        }
        for (bullet_tf, bullet) in leapfrog_bullet_query.iter() {
            let distance = soul_tf.translation.distance(bullet_tf.translation);
            if distance < (soul_radius + bullet_radius) {
                player_state.hp -= bullet.damage as f32;
                player_state.invincibility_timer = player_state.invincibility_duration;
                if player_state.hp <= 0.0 { 
                    player_state.hp = 0.0; 
                    combat_state.mn_fight = MainFightState::PlayerDefeated;
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
                            sprite: Sprite { 
                                image: asset_server.load("texture/heart/spr_heart_0.png"), 
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
                    return;
                }
            }
        }
        for (bullet_tf, bullet) in expr_bullet_query.iter() {
            let distance = soul_tf.translation.distance(bullet_tf.translation);
            if distance < (soul_radius + bullet_radius) {
                player_state.hp -= bullet.damage as f32;
                player_state.invincibility_timer = player_state.invincibility_duration;
                if player_state.hp <= 0.0 { 
                    player_state.hp = 0.0; 
                    combat_state.mn_fight = MainFightState::PlayerDefeated;
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
                            sprite: Sprite { 
                                image: asset_server.load("texture/heart/spr_heart_0.png"), 
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
                    return;
                }
            }
        }
    }
}

// 無敵点滅
pub fn invincibility_update(
    time: Res<Time>,
    mut player_state: ResMut<PlayerState>,
    mut soul_query: Query<&mut Visibility, With<Soul>>,
) {
    if player_state.invincibility_timer > 0.0 {
        player_state.invincibility_timer -= time.delta_secs();

        if let Ok(mut visibility) = soul_query.get_single_mut() {
            if player_state.invincibility_timer <= 0.0 {
                player_state.invincibility_timer = 0.0;
                *visibility = Visibility::Inherited;
            } else {
                let blink_interval = 1.0 / 15.0; 
                let blink_state = (player_state.invincibility_timer / blink_interval).ceil() as i32;
                
                if blink_state % 2 == 0 {
                    *visibility = Visibility::Hidden;
                } else {
                    *visibility = Visibility::Inherited;
                }
            }
        }
    }
}
