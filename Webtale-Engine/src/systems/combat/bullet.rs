use bevy::prelude::*;
use pyo3::prelude::*;
use crate::components::*;
use crate::resources::*;

pub fn leapfrogBulletUpdate(
    mut commands: Commands,
    time: Res<Time>,
    assetServer: Res<AssetServer>,
    mut query: Query<(Entity, &mut Transform, &PythonBullet, &mut Handle<Image>)>,
    _scripts: Res<DanmakuScripts>,
) {
    let dt = time.delta_seconds();
    
    Python::with_gil(|py| {
        for (entity, mut transform, bullet, mut texture) in query.iter_mut() {
            let bulletObj = bullet.bulletData.bind(py);
            
            if let Err(e) = bulletObj.call_method1("sysUpdate", (dt,)) {
                e.print(py);
                continue;
            }
            
            if let Ok(x) = bulletObj.getattr("x").and_then(|v| v.extract::<f32>()) {
                if let Ok(y) = bulletObj.getattr("y").and_then(|v| v.extract::<f32>()) {
                     transform.translation.x = x;
                     transform.translation.y = y;
                }
            }

            if let Ok(textureVal) = bulletObj.getattr("texture") {
                 if !textureVal.is_none() {
                      if let Ok(path) = textureVal.extract::<String>() {
                           *texture = assetServer.load(path);
                      }
                 }
            }
            
            if let Ok(shouldDelete) = bulletObj.getattr("shouldDelete").and_then(|v| v.extract::<bool>()) {
                if shouldDelete {
                    commands.entity(entity).despawn();
                }
            }
        }
    });
}

pub fn soulCollisionDetection(
    mut commands: Commands,
    assetServer: Res<AssetServer>,
    mut gameState: ResMut<GameState>,
    mut soulQuery: Query<(Entity, &Transform), With<Soul>>,
    bulletQuery: Query<(&Transform, &PythonBullet)>,
    mut visibilityParamSet: ParamSet<(
        Query<&mut Visibility, (With<Sprite>, Without<Soul>, Without<EditorWindow>)>,
        Query<&mut Visibility, (With<Text>, Without<Soul>, Without<EditorWindow>)>,
    )>,
) {
    if gameState.invincibilityTimer > 0.0 {
        return;
    }

    if let Ok((soulEntity, soulTf)) = soulQuery.get_single_mut() {
        let soulRadius = 6.0;
        let bulletRadius = 10.0;

        for (bulletTf, bullet) in bulletQuery.iter() {
            let distance = soulTf.translation.distance(bulletTf.translation);
            if distance < (soulRadius + bulletRadius) {
                gameState.hp -= bullet.damage as f32;
                
                gameState.invincibilityTimer = gameState.invincibilityDuration;

                if gameState.hp <= 0.0 { 
                    gameState.hp = 0.0; 
                    gameState.mnFight = 99;

                    for mut visibility in visibilityParamSet.p0().iter_mut() {
                        *visibility = Visibility::Hidden;
                    }
                    for mut visibility in visibilityParamSet.p1().iter_mut() {
                        *visibility = Visibility::Hidden;
                    }

                    commands.entity(soulEntity).despawn();

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
                            texture: assetServer.load("heart/spr_heart_0.png"), 
                            sprite: Sprite { 
                                color: Color::WHITE, 
                                custom_size: Some(Vec2::new(16.0, 16.0)), 
                                ..default() 
                            },
                            transform: Transform::from_translation(Vec3::new(soulTf.translation.x, soulTf.translation.y, 600.0)),
                            ..default()
                        },
                        HeartDefeated {
                            timer: Timer::from_seconds(1.0, TimerMode::Once), 
                            state: HeartDefeatedState::InitialDelay,
                            originalPos: soulTf.translation,
                        },
                        Cleanup,
                    ));
                    
                    break;
                }
            }
        }
    }
}

pub fn invincibilityUpdate(
    time: Res<Time>,
    mut gameState: ResMut<GameState>,
    mut soulQuery: Query<&mut Visibility, With<Soul>>,
) {
    if gameState.invincibilityTimer > 0.0 {
        gameState.invincibilityTimer -= time.delta_seconds();

        if let Ok(mut visibility) = soulQuery.get_single_mut() {
            if gameState.invincibilityTimer <= 0.0 {
                gameState.invincibilityTimer = 0.0;
                *visibility = Visibility::Inherited;
            } else {
                let blinkInterval = 1.0 / 15.0; 
                let blinkState = (gameState.invincibilityTimer / blinkInterval).ceil() as i32;
                
                if blinkState % 2 == 0 {
                    *visibility = Visibility::Hidden;
                } else {
                    *visibility = Visibility::Inherited;
                }
            }
        }
    }
}
