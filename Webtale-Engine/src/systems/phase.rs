use bevy::prelude::Vec2;
use rustpython_vm::builtins::PyDictRef;
use rustpython_vm::compiler::Mode;
use rustpython_vm::convert::ToPyObject;
use rustpython_vm::import::import_codeobj;
use rustpython_vm::PyObjectRef;
use crate::constants::*;
use crate::python_scripts;
use crate::resources::{GameState, PythonRuntime};

pub fn resolveInitialPhase(projectName: &str, requested: &str) -> String {
    if !requested.is_empty() {
        if python_scripts::get_phase_script(projectName, requested).is_some() {
            return requested.to_string();
        }
        println!("Warning: phase script missing projects/{}/phases/{}.py", projectName, requested);
    }

    if python_scripts::get_phase_script(projectName, "phase1").is_some() {
        return "phase1".to_string();
    }

    let mut phaseNames = python_scripts::list_phase_names(projectName);
    phaseNames.sort();
    phaseNames.first().cloned().unwrap_or_default()
}

fn resolveBubbleTextureName(name: &str) -> String {
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

pub fn applyPhaseUpdate(gameState: &mut GameState, projectName: &str, trigger: &str, python_runtime: &PythonRuntime) -> Option<String> {
    if gameState.phaseName.is_empty() {
        return None;
    }

    let phaseName = gameState.phaseName.clone();
    let scriptContent = match python_scripts::get_phase_script(projectName, &phaseName) {
        Some(content) => content,
        None => {
            println!("Warning: phase script missing projects/{}/phases/{}.py", projectName, phaseName);
            return None;
        }
    };

    let apiContent = match python_scripts::get_phase_api_script(projectName) {
        Some(content) => content,
        None => {
            println!("Warning: phase api missing projects/{}/phases/phase_api.py", projectName);
            return None;
        }
    };

    let mut nextPhase: Option<String> = None;

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

        let apiModule = match run_module(apiContent, "phase_api.py", "phase_api") {
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
        if let Err(err) = modules.set_item("phase_api", apiModule.clone(), vm) {
            vm.print_exception(err.clone());
            return;
        }

        let context = vm.ctx.new_dict();
        let _ = context.set_item("turn", vm.new_pyobj(gameState.turnCount), vm);
        let _ = context.set_item("phaseTurn", vm.new_pyobj(gameState.phaseTurn), vm);
        let _ = context.set_item("enemyHp", vm.new_pyobj(gameState.enemyHp), vm);
        let _ = context.set_item("enemyMaxHp", vm.new_pyobj(gameState.enemyMaxHp), vm);
        let _ = context.set_item("enemyName", vm.new_pyobj(gameState.enemyName.clone()), vm);
        let _ = context.set_item("phase", vm.new_pyobj(phaseName.clone()), vm);
        let _ = context.set_item("trigger", vm.new_pyobj(trigger), vm);
        let _ = context.set_item("isFirstTurn", vm.new_pyobj(gameState.turnCount == 1), vm);
        let _ = context.set_item("isPhaseStart", vm.new_pyobj(gameState.phaseTurn == 1), vm);
        let _ = context.set_item("isStart", vm.new_pyobj(trigger == "start"), vm);
        let _ = context.set_item("isTurnStart", vm.new_pyobj(trigger == "turn"), vm);
        let _ = context.set_item("isDamageApplied", vm.new_pyobj(trigger == "damage"), vm);
        let lastAction = if gameState.lastPlayerAction.is_empty() {
            vm.ctx.none()
        } else {
            gameState.lastPlayerAction.clone().to_pyobject(vm)
        };
        let _ = context.set_item("lastPlayerAction", lastAction, vm);
        let lastAct = match &gameState.lastActCommand {
            Some(command) => command.clone().to_pyobject(vm),
            None => vm.ctx.none(),
        };
        let _ = context.set_item("lastActCommand", lastAct, vm);

        match apiModule.get_attr("reset", vm) {
            Ok(resetFunc) => {
                if let Err(err) = vm.invoke(&resetFunc, (context.clone(),)) {
                    vm.print_exception(err.clone());
                }
            }
            Err(err) => {
                vm.print_exception(err.clone());
            }
        }

        let phaseModule = match run_module(scriptContent, &format!("{}.py", phaseName), &phaseName) {
            Some(module) => module,
            None => return,
        };

        let updateFunc = match phaseModule.get_attr("update", vm) {
            Ok(func) => func,
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: phase update missing {:?}", err);
                return;
            }
        };

        let updateResult = match vm.invoke(&updateFunc, (context.clone(),)) {
            Ok(result) => result,
            Err(err) => {
                vm.print_exception(err.clone());
                return;
            }
        };

        let readOptionString = |dict: &PyDictRef, key: &str, label: &str| -> Option<String> {
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

        let readOptionVecString = |dict: &PyDictRef, key: &str, label: &str| -> Option<Vec<String>> {
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

        let readOptionVecF32 = |dict: &PyDictRef, key: &str, label: &str| -> Option<Vec<f32>> {
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

        let mut applyState = |stateDict: &PyDictRef| {
            if let Some(dialogText) = readOptionString(stateDict, "dialogText", "phase") {
                gameState.enemyDialogText = dialogText.clone();
                if gameState.mnFight == 0 && gameState.myFight == 0 && gameState.menuLayer == MENU_LAYER_TOP {
                    gameState.dialogText = dialogText;
                }
            }

            if let Some(attacks) = readOptionVecString(stateDict, "attackPatterns", "phase") {
                gameState.enemyAttacks = attacks;
            }

            if let Some(messages) = readOptionVecString(stateDict, "bubbleMessages", "phase") {
                gameState.enemyBubbleMessages = messages;
            }

            if let Some(message) = readOptionString(stateDict, "bubbleMessage", "phase") {
                if !message.is_empty() {
                    gameState.enemyBubbleMessageOverride = Some(message);
                }
            }

            if let Some(texture) = readOptionString(stateDict, "bubbleTexture", "phase") {
                gameState.enemyBubbleTexture = resolveBubbleTextureName(&texture);
            }

            if let Some(pos) = readOptionVecF32(stateDict, "bubblePosition", "phase") {
                if pos.len() == 2 {
                    gameState.enemyBubblePosOverride = Some(Vec2::new(pos[0], pos[1]));
                } else {
                    println!("Warning: phase bubblePosition invalid");
                }
            }

            nextPhase = readOptionString(stateDict, "nextPhase", "phase");
        };

        if let Ok(dict) = updateResult.try_into_value::<PyDictRef>(vm) {
            applyState(&dict);
            return;
        }

        match apiModule.get_attr("getState", vm) {
            Ok(getState) => match vm.invoke(&getState, ()) {
                Ok(stateResult) => match stateResult.try_into_value::<PyDictRef>(vm) {
                    Ok(dict) => applyState(&dict),
                    Err(err) => vm.print_exception(err),
                },
                Err(err) => vm.print_exception(err),
            },
            Err(err) => vm.print_exception(err),
        }
    });

    nextPhase
}
