use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;
use bevy_egui::EguiContexts;
use pyo3::prelude::*;
use pyo3::types::PyDict; 
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn battle_flow_control(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    game_fonts: Res<GameFonts>,
    _time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>, 
    mut box_res: ResMut<BattleBox>,
    bubbles: Query<Entity, With<SpeechBubble>>,
    bubble_text_query: Query<&Typewriter, With<SpeechBubble>>, 
    mut soul_query: Query<&mut Transform, With<Soul>>,
    mut egui_contexts: EguiContexts,
    editor_query: Query<Entity, With<EditorWindow>>,
) {
    if let Ok(editor_entity) = editor_query.get_single() {
        if egui_contexts.ctx_for_window_mut(editor_entity).wants_keyboard_input() {
            return;
        }
    }

    if game_state.mnfight == 1 {
        if bubbles.is_empty() {
            box_res.target = Rect::new(32.0, 250.0, 602.0, 385.0);
            let bubble_x = 320.0 + 40.0; 
            let bubble_y = 160.0 - 95.0; 
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("blcon/spr_blconsm.png"), 
                    sprite: Sprite { 
                        color: Color::WHITE, 
                        custom_size: Some(Vec2::new(100.0, 80.0)), 
                        anchor: Anchor::TopLeft, 
                        ..default() 
                    },
                    transform: Transform::from_translation(gml_to_bevy(bubble_x, bubble_y) + Vec3::new(0.0, 0.0, Z_BUBBLE)),
                    ..default()
                },
                SpeechBubble,
                Cleanup,
            ));
            let messages = ["Ribbit, ribbit.", "Croak.", "Hop, hop."];
            let msg = messages[rand::thread_rng().gen_range(0..messages.len())];
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("", TextStyle { font: game_fonts.dialog.clone(), font_size: 24.0, color: Color::BLACK }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(bubble_x + 15.0, bubble_y + 15.0) + Vec3::new(0.0, 0.0, Z_BUBBLE_TEXT)),
                    ..default()
                },
                Typewriter { full_text: msg.to_string(), visible_chars: 0, timer: Timer::from_seconds(0.05, TimerMode::Repeating), finished: false },
                SpeechBubble, 
                Cleanup,
            ));
        }
        
        let mut is_finished = false;
        if let Ok(writer) = bubble_text_query.get_single() {
            if writer.finished {
                is_finished = true;
            }
        }

        if is_finished && input.just_pressed(KeyCode::KeyZ) {
            for entity in bubbles.iter() { commands.entity(entity).despawn_recursive(); }
            
            game_state.mnfight = 2; 
            game_state.turntimer = -1.0; 
            
            box_res.target = Rect::new(217.0, 125.0, 417.0, 385.0);
            
            let box_center_x = (217.0 + 417.0) / 2.0;
            let box_center_y = (125.0 + 385.0) / 2.0;
            if let Ok(mut t) = soul_query.get_single_mut() {
                t.translation = gml_to_bevy(box_center_x, box_center_y) + Vec3::new(0.0, 0.0, Z_SOUL);
            }
        }
    }
}

pub fn combat_turn_manager(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut battle_box: ResMut<BattleBox>,
    bullet_query: Query<Entity, With<PythonBullet>>,
    mut scripts: ResMut<DanmakuScripts>,
) {
    if game_state.mnfight == 2 {
        if game_state.turntimer < 0.0 {
            game_state.turntimer = 5.0;
            
            let relative_path = format!("projects/{}/danmaku", PROJECT_NAME);
            let script_file_path = format!("{}/frog_jump.py", relative_path);

            let script_content = std::fs::read_to_string(&script_file_path).unwrap_or_default();
            
            Python::with_gil(|py| {
                let sys = PyModule::import_bound(py, "sys").expect("Failed to import sys");
                let path = sys.getattr("path").expect("Failed to get sys.path");
                
                let env_path = std::env::current_dir().unwrap().join(relative_path);
                let _ = path.call_method1("append", (env_path.to_str().unwrap(),));

                let module = PyModule::from_code_bound(py, &script_content, "frog_jump.py", "frog_jump").expect("Failed to load python script");
                
                let init_func = module.getattr("init").expect("Failed to get init function");
                let init_result = init_func.call0().expect("Failed to call init");
                let init_data: &Bound<PyDict> = init_result.downcast().expect("init should return dict");
                
                let box_data_obj = init_data.get_item("box").expect("box missing").expect("box None");
                let _box_data: Vec<f32> = box_data_obj.extract().expect("box format error");

                let texture_path_obj = init_data.get_item("texture_wait").expect("texture_wait missing").expect("texture_wait None");
                let texture_path: String = texture_path_obj.extract().unwrap();
                
                let spawn_x = ORIGIN_X + battle_box.current.max.x - 40.0;
                let spawn_y = ORIGIN_Y - battle_box.current.max.y + 40.0;

                let spawn_func = module.getattr("spawn").expect("Failed to get spawn function");
                let bullet_obj: PyObject = spawn_func.call0().expect("Failed to call spawn").into();
                
                let bullet_bound = bullet_obj.bind(py);
                let _ = bullet_bound.call_method1("set_pos", (spawn_x, spawn_y));

                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load(texture_path),
                        transform: Transform::from_xyz(spawn_x, spawn_y, 30.0).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    PythonBullet {
                        script_name: "frog_jump".to_string(),
                        bullet_data: bullet_obj,
                        damage: 4,
                    },
                    Cleanup,
                ));
                
                scripts.modules.insert("frog_jump".to_string(), module.into());
            });
        }

        game_state.turntimer -= time.delta_seconds();

        if game_state.turntimer <= 0.0 {
            for entity in bullet_query.iter() {
                commands.entity(entity).despawn();
            }
            
            game_state.mnfight = 3;
            game_state.turntimer = -1.0;
        }
    } else if game_state.mnfight == 3 {
        game_state.mnfight = 0;
        game_state.myfight = 0;
        game_state.menu_layer = 0;
        game_state.dialog_text = "* Froggit hops close!".to_string(); 
        
        battle_box.target = Rect::new(32.0, 250.0, 602.0, 385.0);
    }
}
