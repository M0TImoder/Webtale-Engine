use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;
use crate::components::*;

pub fn heartDefeatedUpdate(
    mut commands: Commands,
    time: Res<Time>,
    assetServer: Res<AssetServer>,
    mut query: Query<(Entity, &mut HeartDefeated, &mut Transform, &mut Handle<Image>)>,
) {
    for (entity, mut defeated, mut transform, mut texture) in query.iter_mut() {
        defeated.timer.tick(time.delta());

        match defeated.state {
            HeartDefeatedState::InitialDelay => {
                if defeated.timer.finished() {
                    defeated.state = HeartDefeatedState::Cracked;
                    defeated.timer = Timer::from_seconds(1.0, TimerMode::Once); 
                    
                    *texture = assetServer.load("heart/spr_heartbreak.png");
                    transform.translation.x -= 2.0; 
                }
            },
            HeartDefeatedState::Cracked => {
                if defeated.timer.finished() {
                    let basePos = transform.translation;
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
                        let directionDeg = rng.gen_range(0.0..360.0);
                        let directionRad = directionDeg * PI / 180.0;
                        let speed = 7.0 * 30.0;
                        
                        let vx = speed * directionRad.cos();
                        let vy = speed * directionRad.sin(); 

                        let shardIndex = rng.gen_range(0..4);
                        let texturePath = format!("heart/spr_heartshards_{}.png", shardIndex);

                        commands.spawn((
                            SpriteBundle {
                                texture: assetServer.load(texturePath),
                                transform: Transform::from_translation(basePos + *offset + Vec3::new(0.0, 0.0, 0.0)).with_translation(Vec3::new(basePos.x + offset.x, basePos.y + offset.y, 600.0)), 
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

pub fn gameOverSequenceUpdate(
    mut commands: Commands,
    time: Res<Time>,
    assetServer: Res<AssetServer>,
    mut query: Query<&mut GameOverSequence>,
    mut logoQuery: Query<&mut Sprite, With<GameOverLogo>>,
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
                            texture: assetServer.load("background/spr_gameoverbg.png"),
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
                for mut sprite in logoQuery.iter_mut() {
                    sprite.color.set_a(alpha);
                }

                if sequence.timer.finished() {
                    sequence.state = GameOverSequenceState::Finished;
                    for mut sprite in logoQuery.iter_mut() {
                        sprite.color.set_a(1.0);
                    }
                }
            },
            GameOverSequenceState::Finished => {
                
            }
        }
    }
}

pub fn heartShardUpdate(
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
