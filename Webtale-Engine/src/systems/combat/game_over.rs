use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;
use crate::components::*;

// ソウル破壊演出
pub fn heart_defeated_update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut HeartDefeated, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut defeated, mut transform, mut sprite) in query.iter_mut() {
        defeated.timer.tick(time.delta());

        match defeated.state {
            HeartDefeatedState::InitialDelay => {
                if defeated.timer.finished() {
                    defeated.state = HeartDefeatedState::Cracked;
                    defeated.timer = Timer::from_seconds(1.0, TimerMode::Once); 
                    
                    sprite.image = asset_server.load("texture/heart/spr_heartbreak.png");
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
                        let texture_path = format!("texture/heart/spr_heartshards_{}.png", shard_index);

                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite { image: asset_server.load(texture_path), ..default() },
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

// ゲームオーバー演出
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
                            sprite: Sprite {
                                image: asset_server.load("texture/background/spr_gameoverbg.png"),
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
                    sprite.color.set_alpha(alpha);
                }

                if sequence.timer.finished() {
                    sequence.state = GameOverSequenceState::Finished;
                    for mut sprite in logo_query.iter_mut() {
                        sprite.color.set_alpha(1.0);
                    }
                }
            },
            GameOverSequenceState::Finished => {
                
            }
        }
    }
}

// ハート破片更新
pub fn heart_shard_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut HeartShard)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut shard) in query.iter_mut() {
        shard.velocity.y -= shard.gravity * dt;
        transform.translation += shard.velocity * dt;

        if transform.translation.y < -300.0 {
            commands.entity(entity).despawn();
        }
    }
}
