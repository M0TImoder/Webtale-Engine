use bevy::prelude::*;
use pyo3::prelude::*;
use crate::components::*;
use crate::resources::*;

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
            let bullet_obj = bullet.bullet_data.bind(py);
            
            if let Err(e) = bullet_obj.call_method1("sys_update", (dt,)) {
                e.print(py);
                continue;
            }
            
            if let Ok(x) = bullet_obj.getattr("x").and_then(|v| v.extract::<f32>()) {
                if let Ok(y) = bullet_obj.getattr("y").and_then(|v| v.extract::<f32>()) {
                     transform.translation.x = x;
                     transform.translation.y = y;
                }
            }

            if let Ok(texture_val) = bullet_obj.getattr("texture") {
                 if !texture_val.is_none() {
                      if let Ok(path) = texture_val.extract::<String>() {
                           *texture = asset_server.load(path);
                      }
                 }
            }
            
            if let Ok(should_delete) = bullet_obj.getattr("should_delete").and_then(|v| v.extract::<bool>()) {
                if should_delete {
                    commands.entity(entity).despawn();
                }
            }
        }
    });
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
