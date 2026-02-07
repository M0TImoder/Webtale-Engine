use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

pub fn leapfrog_bullet_update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    python_runtime: NonSend<PythonRuntime>,
    mut query: Query<(Entity, &mut Transform, &PythonBullet, &mut Handle<Image>)>,
    _scripts: Res<DanmakuScripts>,
) {
    let dt = time.delta_seconds();

    python_runtime.interpreter.enter(|vm| {
        for (entity, mut transform, bullet, mut texture) in query.iter_mut() {
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
                            *texture = asset_server.load(path);
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

pub fn soul_collision_detection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_state: ResMut<PlayerState>,
    mut combat_state: ResMut<CombatState>,
    mut soul_query: Query<(Entity, &Transform), With<Soul>>,
    bullet_query: Query<(&Transform, &PythonBullet)>,
    mut visibility_param_set: ParamSet<(
        Query<&mut Visibility, (With<Sprite>, Without<Soul>, Without<EditorWindow>)>,
        Query<&mut Visibility, (With<Text>, Without<Soul>, Without<EditorWindow>)>,
    )>,
) {
    if player_state.invincibility_timer > 0.0 {
        return;
    }

    if let Ok((soul_entity, soul_tf)) = soul_query.get_single_mut() {
        let soul_radius = 6.0;
        let bullet_radius = 10.0;

        for (bullet_tf, bullet) in bullet_query.iter() {
            let distance = soul_tf.translation.distance(bullet_tf.translation);
            if distance < (soul_radius + bullet_radius) {
                player_state.hp -= bullet.damage as f32;
                
                player_state.invincibility_timer = player_state.invincibility_duration;

                if player_state.hp <= 0.0 { 
                    player_state.hp = 0.0; 
                    combat_state.mn_fight = 99;

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
                            texture: asset_server.load("texture/heart/spr_heart_0.png"), 
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
    mut player_state: ResMut<PlayerState>,
    mut soul_query: Query<&mut Visibility, With<Soul>>,
) {
    if player_state.invincibility_timer > 0.0 {
        player_state.invincibility_timer -= time.delta_seconds();

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
