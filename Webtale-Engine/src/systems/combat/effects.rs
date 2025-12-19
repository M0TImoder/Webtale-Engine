use bevy::prelude::*;
use rand::Rng;
use crate::components::*;

pub fn vaporizeEnemySystem(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Image>>,
    mut query: Query<(Entity, &mut Vaporizing, &mut Sprite, &mut Transform)>,
) {
    let scanSpeed = 100.0; 
    let pixelSize = 2.0;

    for (entity, mut vap, mut sprite, mut transform) in query.iter_mut() {
        let image = if let Some(img) = assets.get(&vap.imageHandle) {
            img
        } else {
            continue;
        };

        let textureWidth = image.texture_descriptor.size.width as f32;
        let textureHeight = image.texture_descriptor.size.height as f32;
        
        let prevLine = vap.scanLine;
        vap.scanLine += scanSpeed * time.delta_seconds();

        let startY = prevLine as u32;
        let endY = (vap.scanLine as u32).min(textureHeight as u32);
        
        let step = if pixelSize < 1.0 { 1 } else { pixelSize as u32 };

        for y in (startY..endY).step_by(step as usize) {
            for x in (0..(textureWidth as u32)).step_by(step as usize) {
                let index = ((y * (textureWidth as u32) + x) * 4) as usize;
                if index + 3 >= image.data.len() { continue; }

                let alpha = image.data[index + 3];

                if alpha > 10 { 
                    let spriteScale = transform.scale.x; 
                    
                    let relativeX = (x as f32 - textureWidth / 2.0) * spriteScale;
                    let relativeY = (textureHeight / 2.0 - y as f32) * spriteScale;

                    let dustPos = Vec3::new(
                        transform.translation.x + relativeX,
                        vap.initialY + relativeY, 
                        0.1
                    );

                    let mut rng = rand::thread_rng();
                    let velocityX = rng.gen_range(-80.0..80.0);
                    let velocityY = rng.gen_range(20.0..80.0);
                    let maxAlpha = rng.gen_range(0.2..1.0);

                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite { 
                                color: Color::rgba(1.0, 1.0, 1.0, maxAlpha), 
                                custom_size: Some(Vec2::splat(pixelSize * spriteScale)), 
                                ..default() 
                            },
                            transform: Transform::from_translation(dustPos),
                            ..default()
                        },
                        DustParticle {
                            velocity: Vec3::new(velocityX, velocityY, 0.0), 
                            timer: Timer::from_seconds(1.0, TimerMode::Once),
                            maxAlpha,
                        },
                        Cleanup,
                    ));
                }
            }
        }

        let currentHeightPx = (textureHeight - vap.scanLine).max(0.0);
        
        if currentHeightPx <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            sprite.rect = Some(Rect {
                min: Vec2::new(0.0, vap.scanLine),
                max: Vec2::new(textureWidth, textureHeight),
            });
            
            let scale = transform.scale.y;
            sprite.custom_size = Some(Vec2::new(textureWidth * scale, currentHeightPx * scale));

            let removedHeight = textureHeight - currentHeightPx;
            transform.translation.y = vap.initialY - (removedHeight * scale / 2.0);
        }
    }
}

pub fn dustParticleUpdate(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut DustParticle)>,
) {
    for (entity, mut transform, mut sprite, mut dust) in query.iter_mut() {
        dust.timer.tick(time.delta());
        
        transform.translation += dust.velocity * time.delta_seconds();

        let alpha = dust.maxAlpha * (1.0 - dust.timer.fraction());
        sprite.color.set_a(alpha);

        if dust.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
