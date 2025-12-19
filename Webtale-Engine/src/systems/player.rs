use bevy::prelude::*;
use bevy_egui::EguiContexts;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn soulPositionSync(
    gameState: Res<GameState>,
    mut soulQuery: Query<&mut Transform, With<Soul>>,
) {
    if (gameState.mnFight != 0 && gameState.mnFight != 2) || gameState.myFight != 0 { 
        if let Ok(mut t) = soulQuery.get_single_mut() {
            t.translation = gml_to_bevy(-200.0, 0.0); 
        }
        return; 
    }
    
    if gameState.mnFight == 2 {
        return;
    }

    let mut transform = soulQuery.single_mut();
    let layer = gameState.menuLayer;
    
    let textStartX = 68.0; 
    let textStartY = 270.0 + 16.0;

    if layer == MENU_LAYER_TOP {
        let offsetX = 8.0 + 8.0; 
        let offsetY = 14.0 + 8.0; 
        let currentBtnIdx = gameState.menuCoords[MENU_LAYER_TOP as usize];
        
        let targetX = match currentBtnIdx {
            0 => BTN_FIGHT_X, 1 => BTN_ACT_X, 2 => BTN_ITEM_X, 3 => BTN_MERCY_X, _ => 0.0,
        };
        let pos = gml_to_bevy(targetX + offsetX, BUTTON_Y_GML + offsetY);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_FIGHT_TARGET || layer == MENU_LAYER_ACT_TARGET {
        let pos = gml_to_bevy(textStartX, textStartY);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_ACT_COMMAND || layer == MENU_LAYER_ITEM {
        let idx = gameState.menuCoords[layer as usize] as usize;
        let col = idx % 2;
        let row = idx / 2;
        let xOffset = if col == 0 { 0.0 } else { 240.0 };
        let yOffset = (row as f32) * 32.0;
        
        let pos = gml_to_bevy(textStartX + xOffset, textStartY + yOffset);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);

    } else if layer == MENU_LAYER_MERCY {
        let idx = gameState.menuCoords[layer as usize] as usize;
        let yOffset = (idx as f32) * 32.0;
        let pos = gml_to_bevy(textStartX, textStartY + yOffset);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);
    }
}

pub fn soulCombatMovement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    gameState: Res<GameState>,
    battleBox: Res<BattleBox>,
    mut query: Query<&mut Transform, With<Soul>>,
    mut eguiContexts: EguiContexts,
    editorQuery: Query<Entity, (With<EditorWindow>, With<Window>)>,
    editorState: Option<Res<EditorState>>,
) {
    if let Ok(editorEntity) = editorQuery.get_single() {
        if eguiContexts.ctx_for_window_mut(editorEntity).wants_keyboard_input() {
            return;
        }
    }

    if let Some(state) = editorState {
        if state.currentTab == EditorTab::DanmakuPreview {
            return;
        }
    }

    if gameState.mnFight != 2 { return; }

    let mut transform = query.single_mut();
    
    let speed = gameState.speed;
    
    let delta = speed * time.delta_seconds();
    let mut moveVec = Vec3::ZERO;

    if input.pressed(KeyCode::ArrowUp)    || input.pressed(KeyCode::KeyW) { moveVec.y += 1.0; }
    if input.pressed(KeyCode::ArrowDown)  || input.pressed(KeyCode::KeyS) { moveVec.y -= 1.0; }
    if input.pressed(KeyCode::ArrowLeft)  || input.pressed(KeyCode::KeyA) { moveVec.x -= 1.0; }
    if input.pressed(KeyCode::ArrowRight) || input.pressed(KeyCode::KeyD) { moveVec.x += 1.0; }

    if moveVec != Vec3::ZERO {
        moveVec = moveVec.normalize() * delta;
        transform.translation += moveVec;
    }

    let soulRadius = 8.0;
    let boxLeft = ORIGIN_X + battleBox.current.min.x + soulRadius;
    let boxRight = ORIGIN_X + battleBox.current.max.x - soulRadius;
    let boxTop = ORIGIN_Y - battleBox.current.min.y - soulRadius;
    let boxBottom = ORIGIN_Y - battleBox.current.max.y + soulRadius;

    transform.translation.x = transform.translation.x.clamp(boxLeft, boxRight);
    transform.translation.y = transform.translation.y.clamp(boxBottom, boxTop);
}
