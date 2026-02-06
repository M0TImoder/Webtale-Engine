use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

pub fn leapfrogBulletUpdate(
    mut commands: Commands,
    time: Res<Time>,
    assetServer: Res<AssetServer>,
    python_runtime: NonSend<PythonRuntime>,
    mut query: Query<(Entity, &mut Transform, &PythonBullet, &mut Handle<Image>)>,
    _scripts: Res<DanmakuScripts>,
) {
    let dt = time.delta_seconds();

    python_runtime.interpreter.enter(|vm| {
        for (entity, mut transform, bullet, mut texture) in query.iter_mut() {
            let bulletObj = bullet.bulletData.clone();

            let sysUpdate = match bulletObj.get_attr("sysUpdate", vm) {
                Ok(func) => func,
                Err(err) => {
                    vm.print_exception(err.clone());
                    continue;
                }
            };
            if let Err(err) = vm.invoke(&sysUpdate, (dt,)) {
                vm.print_exception(err.clone());
                continue;
            }

            let x = match bulletObj.get_attr("x", vm) {
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
            let y = match bulletObj.get_attr("y", vm) {
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

            match bulletObj.get_attr("texture", vm) {
                Ok(textureVal) => match textureVal.try_into_value::<Option<String>>(vm) {
                    Ok(path) => {
                        if let Some(path) = path {
                            *texture = assetServer.load(path);
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

            match bulletObj.get_attr("shouldDelete", vm) {
                Ok(value) => match value.try_into_value::<bool>(vm) {
                    Ok(shouldDelete) => {
                        if shouldDelete {
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
                            texture: assetServer.load("texture/heart/spr_heart_0.png"), 
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
