use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn soul_position_sync(
    game_state: Res<GameState>,
    mut soul_query: Query<&mut Transform, With<Soul>>,
) {
    if game_state.mnfight != 0 && game_state.mnfight != 2 && game_state.myfight == 0 { 
        if let Ok(mut t) = soul_query.get_single_mut() {
            t.translation = gml_to_bevy(-200.0, 0.0); 
        }
        return; 
    }
    
    if game_state.mnfight == 2 || game_state.myfight != 0 {
        return;
    }

    let mut transform = soul_query.single_mut();
    let layer = game_state.menu_layer;
    
    let text_start_x = 68.0; 
    let text_start_y = 270.0 + 16.0;

    if layer == MENU_LAYER_TOP {
        let offset_x = 8.0 + 8.0; 
        let offset_y = 14.0 + 8.0; 
        let current_btn_idx = game_state.menu_coords[MENU_LAYER_TOP as usize];
        
        let target_x = match current_btn_idx {
            0 => BTN_FIGHT_X, 1 => BTN_ACT_X, 2 => BTN_ITEM_X, 3 => BTN_MERCY_X, _ => 0.0,
        };
        let pos = gml_to_bevy(target_x + offset_x, BUTTON_Y_GML + offset_y);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_FIGHT_TARGET || layer == MENU_LAYER_ACT_TARGET {
        let pos = gml_to_bevy(text_start_x, text_start_y);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_ACT_COMMAND || layer == MENU_LAYER_ITEM {
        let idx = game_state.menu_coords[layer as usize] as usize;
        let col = idx % 2;
        let row = idx / 2;
        let x_offset = if col == 0 { 0.0 } else { 240.0 };
        let y_offset = (row as f32) * 32.0;
        
        let pos = gml_to_bevy(text_start_x + x_offset, text_start_y + y_offset);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_MERCY {
        let idx = game_state.menu_coords[layer as usize] as usize;
        let y_offset = (idx as f32) * 32.0;
        let pos = gml_to_bevy(text_start_x, text_start_y + y_offset);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);
    }
}

pub fn soul_combat_movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    game_state: Res<GameState>,
    battle_box: Res<BattleBox>,
    mut query: Query<&mut Transform, With<Soul>>,
) {
    if game_state.mnfight != 2 { return; }

    let mut transform = query.single_mut();
    let speed = 150.0;
    let delta = speed * time.delta_seconds();
    let mut move_vec = Vec3::ZERO;

    if input.pressed(KeyCode::ArrowUp)    || input.pressed(KeyCode::KeyW) { move_vec.y += 1.0; }
    if input.pressed(KeyCode::ArrowDown)  || input.pressed(KeyCode::KeyS) { move_vec.y -= 1.0; }
    if input.pressed(KeyCode::ArrowLeft)  || input.pressed(KeyCode::KeyA) { move_vec.x -= 1.0; }
    if input.pressed(KeyCode::ArrowRight) || input.pressed(KeyCode::KeyD) { move_vec.x += 1.0; }

    if move_vec != Vec3::ZERO {
        move_vec = move_vec.normalize() * delta;
        transform.translation += move_vec;
    }

    let soul_radius = 8.0;
    let box_left = ORIGIN_X + battle_box.current.min.x + soul_radius;
    let box_right = ORIGIN_X + battle_box.current.max.x - soul_radius;
    let box_top = ORIGIN_Y - battle_box.current.min.y - soul_radius;
    let box_bottom = ORIGIN_Y - battle_box.current.max.y + soul_radius;

    transform.translation.x = transform.translation.x.clamp(box_left, box_right);
    transform.translation.y = transform.translation.y.clamp(box_bottom, box_top);
}
