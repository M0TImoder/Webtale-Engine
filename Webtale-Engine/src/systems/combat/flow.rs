use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;
use bevy_egui::EguiContexts;
use rustpython_vm::builtins::PyDictRef;
use rustpython_vm::compiler::Mode;
use rustpython_vm::import::import_codeobj;
use rustpython_vm::PyObjectRef;
use crate::components::*;
use crate::constants::*;
use crate::python_scripts;
use crate::resources::*;
use crate::systems::phase;

pub fn battle_flow_control(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    game_fonts: Res<GameFonts>,
    python_runtime: NonSend<PythonRuntime>,
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

    if game_state.mn_fight == 1 {
        if bubbles.is_empty() {
            game_state.turn_count += 1;
            game_state.phase_turn += 1;
            if let Some(next_phase) = phase::apply_phase_update(&mut game_state, PROJECT_NAME, "turn", &python_runtime) {
                if next_phase != game_state.phase_name {
                    game_state.phase_name = next_phase;
                    game_state.phase_turn = 1;
                    let _ = phase::apply_phase_update(&mut game_state, PROJECT_NAME, "turn", &python_runtime);
                }
            }

            box_res.target = Rect::new(32.0, 250.0, 602.0, 385.0);
            let bubble_pos = game_state.enemy_bubble_pos_override.unwrap_or(Vec2::new(320.0 + 40.0, 160.0 - 95.0));
            let bubble_x = bubble_pos.x; 
            let bubble_y = bubble_pos.y; 
            let bubble_texture = if game_state.enemy_bubble_texture.is_empty() {
                "texture/blcon/spr_blconsm.png".to_string()
            } else {
                game_state.enemy_bubble_texture.clone()
            };
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(bubble_texture), 
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
            let msg = if let Some(message) = game_state.enemy_bubble_message_override.take() {
                message
            } else if game_state.enemy_bubble_messages.is_empty() {
                println!("Warning: enemy bubble messages missing");
                "...".to_string()
            } else {
                let idx = rand::thread_rng().gen_range(0..game_state.enemy_bubble_messages.len());
                game_state.enemy_bubble_messages[idx].clone()
            };
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("", TextStyle { font: game_fonts.dialog.clone(), font_size: 24.0, color: Color::BLACK }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(bubble_x + 15.0, bubble_y + 15.0) + Vec3::new(0.0, 0.0, Z_BUBBLE_TEXT)),
                    ..default()
                },
                Typewriter { full_text: msg, visible_chars: 0, timer: Timer::from_seconds(0.05, TimerMode::Repeating), finished: false },
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
            
            game_state.mn_fight = 2; 
            game_state.turn_timer = -1.0; 
            
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
    python_runtime: NonSend<PythonRuntime>,
    bullet_query: Query<Entity, With<PythonBullet>>,
    mut scripts: ResMut<DanmakuScripts>,
) {
    if game_state.mn_fight == 2 {
        if game_state.turn_timer < 0.0 {
            game_state.turn_timer = 5.0; 
            
            let attack_patterns = &game_state.enemy_attacks;
            let script_name = if !attack_patterns.is_empty() {
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..attack_patterns.len());
                attack_patterns[idx].clone()
            } else {
                println!("Warning: enemyStatus attackPatterns missing");
                "frogJump".to_string() 
            };
            
            let cached_api = scripts.modules.get("api").cloned();
            let cached_module = scripts.modules.get(&script_name).cloned();
            let script_content = if cached_module.is_none() {
                match python_scripts::get_danmaku_script(PROJECT_NAME, &script_name) {
                    Some(content) => Some(content),
                    None => {
                        println!("Warning: script missing projects/{}/danmaku/{}.py", PROJECT_NAME, script_name);
                        return;
                    }
                }
            } else {
                None
            };

            let api_content = if cached_api.is_none() {
                match python_scripts::get_danmaku_api_script(PROJECT_NAME) {
                    Some(content) => Some(content),
                    None => {
                        println!("Warning: script missing projects/{}/danmaku/api.py", PROJECT_NAME);
                        return;
                    }
                }
            } else {
                None
            };

            python_runtime.interpreter.enter(|vm| {
                let run_module = |code: &str, filename: &str, module_name: &str| -> Option<PyObjectRef> {
                    let code_obj = match vm.compile(code, Mode::Exec, filename.to_string()) {
                        Ok(code_obj) => code_obj,
                        Err(err) => {
                            println!("Warning: python compile {} {:?}", filename, err);
                            return None;
                        }
                    };
                    match import_codeobj(vm, module_name, code_obj, true) {
                        Ok(module) => Some(module),
                        Err(err) => {
                            vm.print_exception(err.clone());
                            None
                        }
                    }
                };

                let api_module = match cached_api {
                    Some(module) => module,
                    None => {
                        let api_content = match api_content {
                            Some(content) => content,
                            None => return,
                        };
                        let module = match run_module(api_content, "api.py", "api") {
                            Some(module) => module,
                            None => return,
                        };
                        scripts.modules.insert("api".to_string(), module.clone());
                        module
                    }
                };

                let sys = match vm.import("sys", 0) {
                    Ok(sys) => sys,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let modules = match sys.get_attr("modules", vm) {
                    Ok(modules) => modules,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                if let Err(err) = modules.set_item("api", api_module.clone(), vm) {
                    vm.print_exception(err.clone());
                    return;
                }

                let module = match cached_module {
                    Some(module) => module,
                    None => {
                        let script_content = match script_content {
                            Some(content) => content,
                            None => return,
                        };
                        let module = match run_module(script_content, &format!("{}.py", script_name), &script_name) {
                            Some(module) => module,
                            None => return,
                        };
                        scripts.modules.insert(script_name.clone(), module.clone());
                        module
                    }
                };

                let init_func = match module.get_attr("init", vm) {
                    Ok(func) => func,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let init_result = match vm.invoke(&init_func, ()) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let init_data: PyDictRef = match init_result.try_into_value(vm) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: danmaku init {:?}", err);
                        return;
                    }
                };

                let box_data_obj = match init_data.get_item_opt("box", vm) {
                    Ok(Some(value)) => value,
                    Ok(None) => {
                        println!("Warning: danmaku box missing");
                        return;
                    }
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let _box_data: Vec<f32> = match box_data_obj.try_into_value(vm) {
                    Ok(value) => value,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };

                let texture_path_obj = match init_data.get_item_opt("textureWait", vm) {
                    Ok(Some(value)) => value,
                    Ok(None) => {
                        println!("Warning: danmaku textureWait missing");
                        return;
                    }
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let texture_path: String = match texture_path_obj.try_into_value(vm) {
                    Ok(value) => value,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };

                let spawn_x = ORIGIN_X + battle_box.current.max.x - 40.0;
                let spawn_y = ORIGIN_Y - battle_box.current.max.y + 40.0;

                let spawn_func = match module.get_attr("spawn", vm) {
                    Ok(func) => func,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let bullet_obj: PyObjectRef = match vm.invoke(&spawn_func, ()) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };

                match bullet_obj.get_attr("setPos", vm) {
                    Ok(set_pos) => {
                        if let Err(err) = vm.invoke(&set_pos, (spawn_x, spawn_y)) {
                            vm.print_exception(err.clone());
                        }
                    }
                    Err(err) => {
                        vm.print_exception(err.clone());
                    }
                }

                let damage = match bullet_obj.get_attr("damage", vm) {
                    Ok(value) => match value.try_into_value::<i32>(vm) {
                        Ok(result) => result,
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: bullet damage {:?}", err);
                            0
                        }
                    },
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: bullet damage {:?}", err);
                        0
                    }
                };

                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load(texture_path),
                        transform: Transform::from_xyz(spawn_x, spawn_y, 30.0).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    PythonBullet {
                        script_name: script_name.clone(),
                        bullet_data: bullet_obj.clone(),
                        damage,
                    },
                    Cleanup,
                ));

            });
        }

        game_state.turn_timer -= time.delta_seconds();

        if game_state.turn_timer <= 0.0 {
            for entity in bullet_query.iter() {
                commands.entity(entity).despawn();
            }
            
            game_state.mn_fight = 3;
            game_state.turn_timer = -1.0;
        }
    } else if game_state.mn_fight == 3 {
        game_state.mn_fight = 0;
        game_state.my_fight = 0;
        game_state.menu_layer = 0;
        game_state.dialog_text = game_state.enemy_dialog_text.clone(); 
        
        battle_box.target = Rect::new(32.0, 250.0, 602.0, 385.0);
    }
}
