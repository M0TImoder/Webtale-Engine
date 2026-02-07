use bevy::prelude::*;
use rand::Rng;
use crate::components::*;

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
