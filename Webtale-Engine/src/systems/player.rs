use bevy::prelude::*;
use bevy_egui::EguiContexts;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn soul_position_sync(
    combat_state: Res<CombatState>,
    menu_state: Res<MenuState>,
    mut soul_query: Query<&mut Transform, With<Soul>>,
) {
    if (combat_state.mn_fight != 0 && combat_state.mn_fight != 2) || combat_state.my_fight != 0 { 
        if let Ok(mut t) = soul_query.get_single_mut() {
            t.translation = gml_to_bevy(-200.0, 0.0); 
        }
        return; 
    }
    
    if combat_state.mn_fight == 2 {
        return;
    }

    let mut transform = soul_query.single_mut();
    let layer = menu_state.menu_layer;
    
    let text_start_x = 68.0; 
    let text_start_y = 270.0 + 16.0;

    if layer == MENU_LAYER_TOP {
        let offset_x = 8.0 + 8.0; 
        let offset_y = 14.0 + 8.0; 
        let current_btn_idx = menu_state.menu_coords[MENU_LAYER_TOP as usize];
        
        let target_x = match current_btn_idx {
            0 => BTN_FIGHT_X, 1 => BTN_ACT_X, 2 => BTN_ITEM_X, 3 => BTN_MERCY_X, _ => 0.0,
        };
        let pos = gml_to_bevy(target_x + offset_x, BUTTON_Y_GML + offset_y);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_FIGHT_TARGET || layer == MENU_LAYER_ACT_TARGET {
        let pos = gml_to_bevy(text_start_x, text_start_y);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_ACT_COMMAND || layer == MENU_LAYER_ITEM {
        let idx = menu_state.menu_coords[layer as usize] as usize;
        let col = idx % 2;
        let row = idx / 2;
        let x_offset = if col == 0 { 0.0 } else { 240.0 };
        let y_offset = (row as f32) * 32.0;
        
        let pos = gml_to_bevy(text_start_x + x_offset, text_start_y + y_offset);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_MERCY {
        let idx = menu_state.menu_coords[layer as usize] as usize;
        let y_offset = (idx as f32) * 32.0;
        let pos = gml_to_bevy(text_start_x, text_start_y + y_offset);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);
    }
}

pub fn soul_combat_movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    combat_state: Res<CombatState>,
    player_state: Res<PlayerState>,
    battle_box: Res<BattleBox>,
    mut query: Query<&mut Transform, With<Soul>>,
    mut egui_contexts: EguiContexts,
    editor_query: Query<Entity, (With<EditorWindow>, With<Window>)>,
    editor_state: Option<Res<EditorState>>,
) {
    if let Ok(editor_entity) = editor_query.get_single() {
        if egui_contexts.ctx_for_window_mut(editor_entity).wants_keyboard_input() {
            return;
        }
    }

    if let Some(state) = editor_state {
        if state.current_tab == EditorTab::DanmakuPreview {
            return;
        }
    }

    if combat_state.mn_fight != 2 { return; }

    let mut transform = query.single_mut();
    
    let speed = player_state.speed;
    
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
