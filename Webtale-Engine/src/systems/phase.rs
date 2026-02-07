use bevy::prelude::Vec2;
use rustpython_vm::builtins::PyDictRef;
use rustpython_vm::compiler::Mode;
use rustpython_vm::convert::ToPyObject;
use rustpython_vm::import::import_codeobj;
use rustpython_vm::PyObjectRef;
use crate::constants::*;
use crate::python_scripts;
use crate::python_utils::{read_option_string, read_option_vec_f32, read_option_vec_string};
use crate::resources::{EnemyState, CombatState, MenuState, PythonRuntime, MainFightState, MessageFightState};

// 初期フェーズ取得
fn resolve_initial_phase_from_api(project_name: &str, python_runtime: &PythonRuntime) -> Option<String> {
    let api_content = match python_scripts::get_phase_api_script(project_name) {
        Some(content) => content,
        None => return None,
    };
    let mut initial_phase: Option<String> = None;
    python_runtime.interpreter.enter(|vm| {
        let code_obj = match vm.compile(&api_content, Mode::Exec, "phase_api.py".to_string()) {
            Ok(code_obj) => code_obj,
            Err(err) => {
                println!("Warning: python compile phase_api.py {:?}", err);
                return;
            }
        };
        let api_module = match import_codeobj(vm, "phase_api", code_obj, true) {
            Ok(module) => module,
            Err(err) => {
                vm.print_exception(err.clone());
                return;
            }
        };
        let get_initial = match api_module.get_attr("getInitialPhase", vm) {
            Ok(func) => func,
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: phase api missing getInitialPhase");
                return;
            }
        };
        match vm.invoke(&get_initial, ()) {
            Ok(result) => match result.try_into_value::<String>(vm) {
                Ok(name) => {
                    if name.is_empty() {
                        println!("Warning: phase api getInitialPhase empty");
                    } else {
                        initial_phase = Some(name);
                    }
                }
                Err(err) => {
                    vm.print_exception(err.clone());
                    println!("Warning: phase api getInitialPhase {:?}", err);
                }
            },
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: phase api getInitialPhase {:?}", err);
            }
        }
    });
    initial_phase
}

// 初期フェーズ決定
pub fn resolve_initial_phase(project_name: &str, requested: &str, python_runtime: &PythonRuntime) -> String {
    if !requested.is_empty() {
        if python_scripts::get_phase_script(project_name, requested).is_some() {
            return requested.to_string();
        }
        println!("Warning: phase script missing projects/{}/phases/{}.py", project_name, requested);
    }

    if let Some(initial_phase) = resolve_initial_phase_from_api(project_name, python_runtime) {
        if python_scripts::get_phase_script(project_name, &initial_phase).is_some() {
            return initial_phase;
        }
        println!("Warning: phase script missing projects/{}/phases/{}.py", project_name, initial_phase);
    }

    let mut phase_names = python_scripts::list_phase_names(project_name);
    phase_names.sort();
    phase_names.first().cloned().unwrap_or_default()
}

// 吹き出しテクスチャ解決
fn resolve_bubble_texture_name(name: &str) -> String {
    match name {
        "blconabove" => "texture/blcon/spr_blconabove.png",
        "blconbelow" => "texture/blcon/spr_blconbelow.png",
        "blconsm" => "texture/blcon/spr_blconsm.png",
        "blconsm2" => "texture/blcon/spr_blconsm2.png",
        "blconsm2_shrt" => "texture/blcon/spr_blconsm2_shrt.png",
        "blconsm_plus1" => "texture/blcon/spr_blconsm_plus1.png",
        "blconsm_shrt" => "texture/blcon/spr_blconsm_shrt.png",
        "blcontiny" => "texture/blcon/spr_blcontiny.png",
        "blcontinyabove" => "texture/blcon/spr_blcontinyabove.png",
        "blcontl" => "texture/blcon/spr_blcontl.png",
        "blconwd" => "texture/blcon/spr_blconwd.png",
        "blconwdshrt" => "texture/blcon/spr_blconwdshrt.png",
        "blconwdshrt_l" => "texture/blcon/spr_blconwdshrt_l.png",
        _ => name,
    }
    .to_string()
}

// フェーズ更新
pub fn apply_phase_update(enemy_state: &mut EnemyState, combat_state: &mut CombatState, menu_state: &mut MenuState, project_name: &str, trigger: &str, python_runtime: &PythonRuntime) -> Option<String> {
    if combat_state.phase_name.is_empty() {
        return None;
    }

    let phase_name = combat_state.phase_name.clone();
    let script_content = match python_scripts::get_phase_script(project_name, &phase_name) {
        Some(content) => content,
        None => {
            println!("Warning: phase script missing projects/{}/phases/{}.py", project_name, phase_name);
            return None;
        }
    };

    let api_content = match python_scripts::get_phase_api_script(project_name) {
        Some(content) => content,
        None => {
            println!("Warning: phase api missing projects/{}/phases/phase_api.py", project_name);
            return None;
        }
    };

    let mut next_phase: Option<String> = None;

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

        let api_module = match run_module(&api_content, "phase_api.py", "phase_api") {
            Some(module) => module,
            None => return,
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
        if let Err(err) = modules.set_item("phase_api", api_module.clone(), vm) {
            vm.print_exception(err.clone());
            return;
        }

        let context = vm.ctx.new_dict();
        let _ = context.set_item("turn", vm.new_pyobj(combat_state.turn_count), vm);
        let _ = context.set_item("phaseTurn", vm.new_pyobj(combat_state.phase_turn), vm);
        let _ = context.set_item("enemyHp", vm.new_pyobj(enemy_state.hp), vm);
        let _ = context.set_item("enemyMaxHp", vm.new_pyobj(enemy_state.max_hp), vm);
        let _ = context.set_item("enemyName", vm.new_pyobj(enemy_state.name.clone()), vm);
        let _ = context.set_item("phase", vm.new_pyobj(phase_name.clone()), vm);
        let _ = context.set_item("trigger", vm.new_pyobj(trigger), vm);
        let _ = context.set_item("isFirstTurn", vm.new_pyobj(combat_state.turn_count == 1), vm);
        let _ = context.set_item("isPhaseStart", vm.new_pyobj(combat_state.phase_turn == 1), vm);
        let _ = context.set_item("isStart", vm.new_pyobj(trigger == "start"), vm);
        let _ = context.set_item("isTurnStart", vm.new_pyobj(trigger == "turn"), vm);
        let _ = context.set_item("isDamageApplied", vm.new_pyobj(trigger == "damage"), vm);
        let last_action = if combat_state.last_player_action.is_empty() {
            vm.ctx.none()
        } else {
            combat_state.last_player_action.clone().to_pyobject(vm)
        };
        let _ = context.set_item("lastPlayerAction", last_action, vm);
        let last_act = match &combat_state.last_act_command {
            Some(command) => command.clone().to_pyobject(vm),
            None => vm.ctx.none(),
        };
        let _ = context.set_item("lastActCommand", last_act, vm);

        match api_module.get_attr("reset", vm) {
            Ok(reset_func) => {
                if let Err(err) = vm.invoke(&reset_func, (context.clone(),)) {
                    vm.print_exception(err.clone());
                }
            }
            Err(err) => {
                vm.print_exception(err.clone());
            }
        }

        let phase_module = match run_module(&script_content, &format!("{}.py", phase_name), &phase_name) {
            Some(module) => module,
            None => return,
        };

        let update_func = match phase_module.get_attr("update", vm) {
            Ok(func) => func,
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: phase update missing {:?}", err);
                return;
            }
        };

        let update_result = match vm.invoke(&update_func, (context.clone(),)) {
            Ok(result) => result,
            Err(err) => {
                vm.print_exception(err.clone());
                return;
            }
        };

        let mut apply_state = |state_dict: &PyDictRef| {
            if let Some(dialog_text) = read_option_string(vm, state_dict, "dialogText", "phase", false) {
                enemy_state.dialog_text = dialog_text.clone();
                if combat_state.mn_fight == MainFightState::Menu && combat_state.my_fight == MessageFightState::None && menu_state.menu_layer == MENU_LAYER_TOP {
                    menu_state.dialog_text = dialog_text;
                }
            }

            if let Some(attacks) = read_option_vec_string(vm, state_dict, "attackPatterns", "phase", false) {
                enemy_state.attacks = attacks;
            }

            if let Some(messages) = read_option_vec_string(vm, state_dict, "bubbleMessages", "phase", false) {
                enemy_state.bubble_messages = messages;
            }

            if let Some(message) = read_option_string(vm, state_dict, "bubbleMessage", "phase", false) {
                if !message.is_empty() {
                    enemy_state.bubble_message_override = Some(message);
                }
            }

            if let Some(texture) = read_option_string(vm, state_dict, "bubbleTexture", "phase", false) {
                enemy_state.bubble_texture = resolve_bubble_texture_name(&texture);
            }

            if let Some(pos) = read_option_vec_f32(vm, state_dict, "bubblePosition", "phase", false) {
                if pos.len() == 2 {
                    enemy_state.bubble_pos_override = Some(Vec2::new(pos[0], pos[1]));
                } else {
                    println!("Warning: phase bubblePosition invalid");
                }
            }

            next_phase = read_option_string(vm, state_dict, "nextPhase", "phase", false);
        };

        if let Ok(dict) = update_result.try_into_value::<PyDictRef>(vm) {
            apply_state(&dict);
            return;
        }

        match api_module.get_attr("getState", vm) {
            Ok(get_state) => match vm.invoke(&get_state, ()) {
                Ok(state_result) => match state_result.try_into_value::<PyDictRef>(vm) {
                    Ok(dict) => apply_state(&dict),
                    Err(err) => vm.print_exception(err),
                },
                Err(err) => vm.print_exception(err),
            },
            Err(err) => vm.print_exception(err),
        }
    });

    next_phase
}
