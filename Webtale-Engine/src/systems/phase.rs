use bevy::prelude::Vec2;
use rustpython_vm::builtins::PyDictRef;
use rustpython_vm::compiler::Mode;
use rustpython_vm::convert::ToPyObject;
use rustpython_vm::import::import_codeobj;
use rustpython_vm::PyObjectRef;
use crate::constants::*;
use crate::python_scripts;
use crate::resources::{GameState, PythonRuntime};

pub fn resolve_initial_phase(project_name: &str, requested: &str) -> String {
    if !requested.is_empty() {
        if python_scripts::get_phase_script(project_name, requested).is_some() {
            return requested.to_string();
        }
        println!("Warning: phase script missing projects/{}/phases/{}.py", project_name, requested);
    }

    if python_scripts::get_phase_script(project_name, "phase1").is_some() {
        return "phase1".to_string();
    }

    let mut phase_names = python_scripts::list_phase_names(project_name);
    phase_names.sort();
    phase_names.first().cloned().unwrap_or_default()
}

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

pub fn apply_phase_update(game_state: &mut GameState, project_name: &str, trigger: &str, python_runtime: &PythonRuntime) -> Option<String> {
    if game_state.phase_name.is_empty() {
        return None;
    }

    let phase_name = game_state.phase_name.clone();
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

        let api_module = match run_module(api_content, "phase_api.py", "phase_api") {
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
        let _ = context.set_item("turn", vm.new_pyobj(game_state.turn_count), vm);
        let _ = context.set_item("phaseTurn", vm.new_pyobj(game_state.phase_turn), vm);
        let _ = context.set_item("enemyHp", vm.new_pyobj(game_state.enemy_hp), vm);
        let _ = context.set_item("enemyMaxHp", vm.new_pyobj(game_state.enemy_max_hp), vm);
        let _ = context.set_item("enemyName", vm.new_pyobj(game_state.enemy_name.clone()), vm);
        let _ = context.set_item("phase", vm.new_pyobj(phase_name.clone()), vm);
        let _ = context.set_item("trigger", vm.new_pyobj(trigger), vm);
        let _ = context.set_item("isFirstTurn", vm.new_pyobj(game_state.turn_count == 1), vm);
        let _ = context.set_item("isPhaseStart", vm.new_pyobj(game_state.phase_turn == 1), vm);
        let _ = context.set_item("isStart", vm.new_pyobj(trigger == "start"), vm);
        let _ = context.set_item("isTurnStart", vm.new_pyobj(trigger == "turn"), vm);
        let _ = context.set_item("isDamageApplied", vm.new_pyobj(trigger == "damage"), vm);
        let last_action = if game_state.last_player_action.is_empty() {
            vm.ctx.none()
        } else {
            game_state.last_player_action.clone().to_pyobject(vm)
        };
        let _ = context.set_item("lastPlayerAction", last_action, vm);
        let last_act = match &game_state.last_act_command {
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

        let phase_module = match run_module(script_content, &format!("{}.py", phase_name), &phase_name) {
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

        let read_option_string = |dict: &PyDictRef, key: &str, label: &str| -> Option<String> {
            match dict.get_item_opt(key, vm) {
                Ok(Some(value)) => match value.try_into_value::<Option<String>>(vm) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: {} {} {:?}", label, key, err);
                        None
                    }
                },
                Ok(None) => None,
                Err(err) => {
                    vm.print_exception(err.clone());
                    println!("Warning: {} {} {:?}", label, key, err);
                    None
                }
            }
        };

        let read_option_vec_string = |dict: &PyDictRef, key: &str, label: &str| -> Option<Vec<String>> {
            match dict.get_item_opt(key, vm) {
                Ok(Some(value)) => match value.try_into_value::<Option<Vec<String>>>(vm) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: {} {} {:?}", label, key, err);
                        None
                    }
                },
                Ok(None) => None,
                Err(err) => {
                    vm.print_exception(err.clone());
                    println!("Warning: {} {} {:?}", label, key, err);
                    None
                }
            }
        };

        let read_option_vec_f32 = |dict: &PyDictRef, key: &str, label: &str| -> Option<Vec<f32>> {
            match dict.get_item_opt(key, vm) {
                Ok(Some(value)) => match value.try_into_value::<Option<Vec<f32>>>(vm) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: {} {} {:?}", label, key, err);
                        None
                    }
                },
                Ok(None) => None,
                Err(err) => {
                    vm.print_exception(err.clone());
                    println!("Warning: {} {} {:?}", label, key, err);
                    None
                }
            }
        };

        let mut apply_state = |state_dict: &PyDictRef| {
            if let Some(dialog_text) = read_option_string(state_dict, "dialogText", "phase") {
                game_state.enemy_dialog_text = dialog_text.clone();
                if game_state.mn_fight == 0 && game_state.my_fight == 0 && game_state.menu_layer == MENU_LAYER_TOP {
                    game_state.dialog_text = dialog_text;
                }
            }

            if let Some(attacks) = read_option_vec_string(state_dict, "attackPatterns", "phase") {
                game_state.enemy_attacks = attacks;
            }

            if let Some(messages) = read_option_vec_string(state_dict, "bubbleMessages", "phase") {
                game_state.enemy_bubble_messages = messages;
            }

            if let Some(message) = read_option_string(state_dict, "bubbleMessage", "phase") {
                if !message.is_empty() {
                    game_state.enemy_bubble_message_override = Some(message);
                }
            }

            if let Some(texture) = read_option_string(state_dict, "bubbleTexture", "phase") {
                game_state.enemy_bubble_texture = resolve_bubble_texture_name(&texture);
            }

            if let Some(pos) = read_option_vec_f32(state_dict, "bubblePosition", "phase") {
                if pos.len() == 2 {
                    game_state.enemy_bubble_pos_override = Some(Vec2::new(pos[0], pos[1]));
                } else {
                    println!("Warning: phase bubblePosition invalid");
                }
            }

            next_phase = read_option_string(state_dict, "nextPhase", "phase");
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
