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

// 戦闘フロー
pub fn battle_flow_control(
    mut commands: Commands,
    mut enemy_state: ResMut<EnemyState>,
    mut combat_state: ResMut<CombatState>,
    mut menu_state: ResMut<MenuState>,
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
){
    if let Ok(editor_entity) = editor_query.get_single() {
        if egui_contexts.ctx_for_entity_mut(editor_entity).wants_keyboard_input() {
            return;
        }
    }

    if combat_state.mn_fight == MainFightState::EnemyDialog {
        if bubbles.is_empty() {
            combat_state.turn_count += 1;
            combat_state.phase_turn += 1;
            if let Some(next_phase) = phase::apply_phase_update(&mut enemy_state, &mut combat_state, &mut menu_state, PROJECT_NAME, "turn", &python_runtime) {
                if next_phase != combat_state.phase_name {
                    combat_state.phase_name = next_phase;
                    combat_state.phase_turn = 1;
                    let _ = phase::apply_phase_update(&mut enemy_state, &mut combat_state, &mut menu_state, PROJECT_NAME, "turn", &python_runtime);
                }
            }

            box_res.target = Rect::new(32.0, 250.0, 602.0, 385.0);
            let bubble_pos = enemy_state.bubble_pos_override.unwrap_or(Vec2::new(320.0 + 40.0, 160.0 - 95.0));
            let bubble_x = bubble_pos.x; 
            let bubble_y = bubble_pos.y; 
            let bubble_texture = if enemy_state.bubble_texture.is_empty() {
                "texture/blcon/spr_blconsm.png".to_string()
            } else {
                enemy_state.bubble_texture.clone()
            };
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite { 
                        image: asset_server.load(bubble_texture),
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
            let msg = if let Some(message) = enemy_state.bubble_message_override.take() {
                message
            } else if enemy_state.bubble_messages.is_empty() {
                println!("Warning: enemy bubble messages missing");
                "...".to_string()
            } else {
                let idx = rand::thread_rng().gen_range(0..enemy_state.bubble_messages.len());
                enemy_state.bubble_messages[idx].clone()
            };
            commands.spawn((
                Text2d::new(""),
                TextFont { font: game_fonts.dialog.clone(), font_size: 24.0 * TEXT_SCALE, ..default() },
                TextColor(Color::BLACK),
                Anchor::TopLeft,
                Transform::from_translation(gml_to_bevy(bubble_x + 15.0, bubble_y + 15.0) + Vec3::new(0.0, 0.0, Z_BUBBLE_TEXT)),
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
            
            combat_state.mn_fight = MainFightState::EnemyAttack; 
            combat_state.turn_timer = -1.0; 
            
            box_res.target = Rect::new(217.0, 125.0, 417.0, 385.0);
            
            let box_center_x = (217.0 + 417.0) / 2.0;
            let box_center_y = (125.0 + 385.0) / 2.0;
            if let Ok(mut t) = soul_query.get_single_mut() {
                t.translation = gml_to_bevy(box_center_x, box_center_y) + Vec3::new(0.0, 0.0, Z_SOUL);
            }
        }
    }
}

// 弾幕ターン管理
pub fn combat_turn_manager(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    enemy_state: Res<EnemyState>,
    mut combat_state: ResMut<CombatState>,
    mut menu_state: ResMut<MenuState>,
    mut battle_box: ResMut<BattleBox>,
    python_runtime: NonSend<PythonRuntime>,
    bullet_query: Query<Entity, With<PythonBullet>>,
    mut scripts: ResMut<DanmakuScripts>,
) {
    if combat_state.mn_fight == MainFightState::EnemyAttack {
        if combat_state.turn_timer < 0.0 {
            combat_state.turn_timer = 5.0; 
            
            let attack_patterns = &enemy_state.attacks;
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
                        let api_content = match api_content.as_deref() {
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
                        let script_content = match script_content.as_deref() {
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
                        sprite: Sprite { image: asset_server.load(texture_path), ..default() },
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

        combat_state.turn_timer -= time.delta_secs();

        if combat_state.turn_timer <= 0.0 {
            for entity in bullet_query.iter() {
                commands.entity(entity).despawn();
            }
            
            combat_state.mn_fight = MainFightState::TurnCleanup;
            combat_state.turn_timer = -1.0;
        }
    } else if combat_state.mn_fight == MainFightState::TurnCleanup {
        combat_state.mn_fight = MainFightState::Menu;
        combat_state.my_fight = MessageFightState::None;
        menu_state.menu_layer = 0;
        menu_state.dialog_text = enemy_state.dialog_text.clone(); 
        
        battle_box.target = Rect::new(32.0, 250.0, 602.0, 385.0);
    }
}
