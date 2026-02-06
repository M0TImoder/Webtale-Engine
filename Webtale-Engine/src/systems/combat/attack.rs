use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_egui::EguiContexts;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;
use crate::systems::phase;

pub fn attackBarUpdate(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut gameState: ResMut<GameState>,
    assetServer: Res<AssetServer>,
    mut query: Query<(Entity, &mut Transform, &mut AttackBar, &mut Handle<Image>)>,
    enemyQuery: Query<&Transform, (With<EnemyBody>, Without<AttackBar>)>,
    mut eguiContexts: EguiContexts,
    editorQuery: Query<Entity, With<EditorWindow>>,
) {
    if let Ok(editorEntity) = editorQuery.get_single() {
        if eguiContexts.ctx_for_window_mut(editorEntity).wants_keyboard_input() {
            return;
        }
    }

    if gameState.mnFight != 4 && gameState.mnFight != 5 { return; }

    for (barEntity, mut transform, mut bar, mut texture) in query.iter_mut() {
        if bar.moving {
            transform.translation.x += bar.speed * time.delta_seconds();
            
            if bar.flashState {
                 bar.flashState = false;
                 *texture = assetServer.load("texture/attack/spr_targetchoice_0.png");
            }

            let boxCenterX = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 0.0).x;
            let missThreshold = boxCenterX + 280.0;
            let autoPress = transform.translation.x > missThreshold;

            if input.just_pressed(KeyCode::KeyZ) || autoPress {
                if autoPress {
                    commands.entity(barEntity).despawn();
                } else {
                    bar.moving = false;
                    *texture = assetServer.load("texture/attack/spr_targetchoice_1.png");
                    bar.flashState = true; 
                }
                
                let distance = (transform.translation.x - boxCenterX).abs();
                
                let baseDamage = gameState.attack;
                
                let damage = if distance < 12.0 {
                    (baseDamage * 2.2) as i32 
                } else {
                    let stretch = (280.0 - distance).max(0.0) / 280.0;
                    (baseDamage * stretch * 2.0) as i32
                };

                let enemyPos = if let Ok(eTrans) = enemyQuery.get_single() {
                    eTrans.translation
                } else {
                    gml_to_bevy(320.0, 136.0)
                };

                let waitTime = if damage > 0 {
                    commands.spawn((
                        SpriteBundle {
                            texture: assetServer.load("texture/attack/spr_strike_0.png"),
                            transform: Transform {
                                translation: enemyPos + Vec3::new(0.0, 0.0, Z_SLICE),
                                scale: Vec3::splat(2.0),
                                ..default()
                            },
                            ..default()
                        },
                        SliceEffect { timer: Timer::from_seconds(0.15, TimerMode::Repeating), frameIndex: 0 },
                        Cleanup,
                    ));
                    0.9 
                } else {
                    0.0
                };

                commands.spawn(PendingDamage {
                    timer: Timer::from_seconds(waitTime, TimerMode::Once),
                    damage,
                    targetPos: enemyPos,
                });

                gameState.mnFight = 5; 
            }
        } else {
            if bar.flashTimer.tick(time.delta()).just_finished() {
                bar.flashState = !bar.flashState;
                let path = if bar.flashState { "texture/attack/spr_targetchoice_1.png" } else { "texture/attack/spr_targetchoice_0.png" };
                *texture = assetServer.load(path);
            }
        }
    }
}

pub fn applyPendingDamage(
    mut commands: Commands,
    time: Res<Time>,
    mut gameState: ResMut<GameState>,
    assetServer: Res<AssetServer>,
    _gameFonts: Res<GameFonts>,
    mut query: Query<(Entity, &mut PendingDamage)>,
) {
    for (entity, mut pending) in query.iter_mut() {
        if pending.timer.tick(time.delta()).finished() {
            let oldHp = gameState.enemyHp;
            gameState.enemyHp = (gameState.enemyHp - pending.damage).max(0);
            let damage = pending.damage;
            let enemyPos = pending.targetPos;
            gameState.lastActCommand = None;
            gameState.lastPlayerAction = if damage > 0 {
                "attackHit".to_string()
            } else {
                "attackMiss".to_string()
            };
            if let Some(nextPhase) = phase::applyPhaseUpdate(&mut gameState, PROJECT_NAME, "damage") {
                if nextPhase != gameState.phaseName {
                    gameState.phaseName = nextPhase;
                    gameState.phaseTurn = 0;
                }
            }

            let textStartPos = enemyPos + Vec3::new(0.0, 50.0, Z_DAMAGE_TEXT);

            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(textStartPos),
                    ..default()
                },
                DamageNumber { 
                    timer: Timer::from_seconds(1.2, TimerMode::Once),
                    velocityY: 240.0, 
                    gravity: 900.0,
                    startY: textStartPos.y,
                },
                Cleanup,
            )).with_children(|parent| {
                let scale = 1.25;

                if damage > 0 {
                    let dmgStr = format!("{}", damage);
                    let charSpacing = 42.0; 

                    let totalWidth = (dmgStr.chars().count() as f32) * charSpacing;
                    let startXOffset = -(totalWidth / 2.0) + (charSpacing / 2.0);

                    for (i, char) in dmgStr.chars().enumerate() {
                        let charX = startXOffset + (i as f32 * charSpacing);
                        let texturePath = format!("texture/dmgnum/spr_dmgnum_o_{}.png", char);

                        parent.spawn(SpriteBundle {
                            texture: assetServer.load(texturePath),
                            sprite: Sprite { 
                                color: Color::rgb(0.8, 0.0, 0.0), 
                                custom_size: None,
                                ..default() 
                            },
                            transform: Transform::from_xyz(charX, 0.0, 0.0).with_scale(Vec3::splat(scale)),
                            ..default()
                        });
                    }
                } else {
                    parent.spawn(SpriteBundle {
                        texture: assetServer.load("texture/dmgnum/spr_dmgmiss_o.png"),
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
                let barWidthMax = 140.0;
                let barHeight = 14.0;
                let barPos = enemyPos + Vec3::new(0.0, 20.0, Z_DAMAGE_HP_BAR);

                commands.spawn((
                    SpatialBundle {
                        transform: Transform::from_translation(barPos),
                        ..default()
                    },
                    EnemyHpBar {
                        lifespan: Timer::from_seconds(1.2, TimerMode::Once),
                        animation: Timer::from_seconds(1.0, TimerMode::Once),
                        startWidth: (oldHp as f32 / gameState.enemyMaxHp as f32) * barWidthMax,
                        targetWidth: (gameState.enemyHp as f32 / gameState.enemyMaxHp as f32) * barWidthMax,
                    },
                    Cleanup,
                )).with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite { color: Color::DARK_GRAY, custom_size: Some(Vec2::new(barWidthMax, barHeight)), ..default() },
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)), 
                        ..default()
                    });
                    let leftOffset = -barWidthMax / 2.0;
                    parent.spawn((
                        SpriteBundle {
                            sprite: Sprite { 
                                color: Color::rgb(0.0, 1.0, 0.0), 
                                custom_size: Some(Vec2::new((oldHp as f32 / gameState.enemyMaxHp as f32) * barWidthMax, barHeight)),
                                anchor: Anchor::CenterLeft, 
                                ..default() 
                            },
                            transform: Transform::from_translation(Vec3::new(leftOffset, 0.0, 0.0)),
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

pub fn animateSliceEffect(
    mut commands: Commands,
    time: Res<Time>,
    assetServer: Res<AssetServer>,
    mut query: Query<(Entity, &mut SliceEffect, &mut Handle<Image>)>,
) {
    for (entity, mut effect, mut texture) in query.iter_mut() {
        if effect.timer.tick(time.delta()).just_finished() {
            effect.frameIndex += 1;
            if effect.frameIndex > 5 {
                commands.entity(entity).despawn();
            } else {
                let path = format!("texture/attack/spr_strike_{}.png", effect.frameIndex);
                *texture = assetServer.load(path);
            }
        }
    }
}

pub fn damageNumberUpdate(
    mut commands: Commands,
    time: Res<Time>,
    mut gameState: ResMut<GameState>,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber), Without<EnemyBody>>,
    attackBarQuery: Query<Entity, With<AttackBar>>,
    targetBoxQuery: Query<Entity, With<AttackTargetBox>>,
    mut enemyQuery: Query<(Entity, &mut Sprite, &Transform, &Handle<Image>), With<EnemyBody>>,
) {
    for (entity, mut transform, mut dmg) in query.iter_mut() {
        dmg.timer.tick(time.delta());
        
        transform.translation.y += dmg.velocityY * time.delta_seconds();
        dmg.velocityY -= dmg.gravity * time.delta_seconds();

        if transform.translation.y < dmg.startY {
            transform.translation.y = dmg.startY;
            dmg.velocityY = 0.0;
            dmg.gravity = 0.0;
        }

        if dmg.timer.finished() {
            commands.entity(entity).despawn_recursive();
            
            for barEntity in attackBarQuery.iter() { commands.entity(barEntity).despawn(); }
            for boxEntity in targetBoxQuery.iter() { commands.entity(boxEntity).despawn(); }
            
            if gameState.enemyHp <= 0 {
                for (eEntity, _, eTransform, handle) in enemyQuery.iter_mut() {
                    commands.entity(eEntity).insert(Vaporizing {
                        scanLine: 0.0,
                        imageHandle: handle.clone(),
                        initialY: eTransform.translation.y,
                    });
                }
                gameState.mnFight = 0; 
            } else {
                gameState.mnFight = 1; 
                gameState.bubbleTimer.reset(); 
                gameState.menuLayer = MENU_LAYER_TOP;
            }
        }
    }
}

pub fn enemyHpBarUpdate(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EnemyHpBar, &Children)>,
    mut barSpriteQuery: Query<&mut Sprite, With<EnemyHpBarForeground>>,
) {
    for (entity, mut bar, children) in query.iter_mut() {
        bar.lifespan.tick(time.delta());
        bar.animation.tick(time.delta());

        let t = bar.animation.fraction();
        let currentWidth = bar.startWidth + (bar.targetWidth - bar.startWidth) * t;

        for &child in children.iter() {
            if let Ok(mut sprite) = barSpriteQuery.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(currentWidth, 14.0));
            }
        }

        if bar.lifespan.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
