use bevy::prelude::Vec2;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::fs;
use crate::constants::*;
use crate::resources::GameState;

pub fn resolveInitialPhase(projectName: &str, requested: &str) -> String {
    let relativePath = format!("projects/{}/phases", projectName);
    if !requested.is_empty() {
        let requestedPath = format!("{}/{}.py", relativePath, requested);
        if fs::metadata(&requestedPath).is_ok() {
            return requested.to_string();
        }
        println!("Warning: phase script missing {}", requestedPath);
    }

    let phase1Path = format!("{}/phase1.py", relativePath);
    if fs::metadata(&phase1Path).is_ok() {
        return "phase1".to_string();
    }

    let entries = match fs::read_dir(&relativePath) {
        Ok(entries) => entries,
        Err(_) => return String::new(),
    };

    let mut phaseNames: Vec<String> = vec![];
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("wep") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|value| value.to_str()) {
            phaseNames.push(stem.to_string());
        }
    }
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

pub fn applyPhaseUpdate(gameState: &mut GameState, projectName: &str, trigger: &str) -> Option<String> {
    if gameState.phaseName.is_empty() {
        return None;
    }

    let phaseName = gameState.phaseName.clone();
    let relativePath = format!("projects/{}/phases", projectName);
    let scriptPath = format!("{}/{}.py", relativePath, phaseName);
    let apiPath = format!("{}/phase_api.py", relativePath);

    let scriptContent = match fs::read_to_string(&scriptPath) {
        Ok(content) => content,
        Err(err) => {
            println!("Warning: phase script load {} {}", scriptPath, err);
            return None;
        }
    };

    let apiContent = match fs::read_to_string(&apiPath) {
        Ok(content) => content,
        Err(err) => {
            println!("Warning: phase api load {} {}", apiPath, err);
            return None;
        }
    };

    let mut nextPhase: Option<String> = None;

    Python::with_gil(|py| {
        let sys = match PyModule::import_bound(py, "sys") {
            Ok(sys) => sys,
            Err(err) => {
                err.print(py);
                return;
            }
        };
        let path = match sys.getattr("path") {
            Ok(path) => path,
            Err(err) => {
                err.print(py);
                return;
            }
        };
        if let Ok(envPath) = std::env::current_dir().map(|dir| dir.join(&relativePath)) {
            if let Some(envStr) = envPath.to_str() {
                let _ = path.call_method1("append", (envStr,));
            }
        }

        let apiModule = match PyModule::from_code_bound(py, &apiContent, "phase_api.py", "phase_api") {
            Ok(module) => module,
            Err(err) => {
                err.print(py);
                return;
            }
        };

        let modules = match sys.getattr("modules") {
            Ok(modules) => modules,
            Err(err) => {
                err.print(py);
                return;
            }
        };
        if let Err(err) = modules.set_item("phase_api", &apiModule) {
            err.print(py);
            return;
        }

        let context = PyDict::new_bound(py);
        let _ = context.set_item("turn", gameState.turnCount);
        let _ = context.set_item("phaseTurn", gameState.phaseTurn);
        let _ = context.set_item("enemyHp", gameState.enemyHp);
        let _ = context.set_item("enemyMaxHp", gameState.enemyMaxHp);
        let _ = context.set_item("enemyName", gameState.enemyName.clone());
        let _ = context.set_item("phase", phaseName.clone());
        let _ = context.set_item("trigger", trigger);
        let _ = context.set_item("isFirstTurn", gameState.turnCount == 1);
        let _ = context.set_item("isPhaseStart", gameState.phaseTurn == 1);
        let _ = context.set_item("isStart", trigger == "start");
        let _ = context.set_item("isTurnStart", trigger == "turn");
        let _ = context.set_item("isDamageApplied", trigger == "damage");
        let lastAction = if gameState.lastPlayerAction.is_empty() {
            None
        } else {
            Some(gameState.lastPlayerAction.clone())
        };
        let _ = context.set_item("lastPlayerAction", lastAction);
        let _ = context.set_item("lastActCommand", gameState.lastActCommand.clone());

        if let Ok(resetFunc) = apiModule.getattr("reset") {
            if let Err(err) = resetFunc.call1((&context,)) {
                err.print(py);
            }
        }

        let phaseModule = match PyModule::from_code_bound(py, &scriptContent, &format!("{}.py", phaseName), &phaseName) {
            Ok(module) => module,
            Err(err) => {
                err.print(py);
                return;
            }
        };

        let updateFunc = match phaseModule.getattr("update") {
            Ok(func) => func,
            Err(err) => {
                println!("Warning: phase update missing {}", err);
                return;
            }
        };

        let updateResult = match updateFunc.call1((&context,)) {
            Ok(result) => result,
            Err(err) => {
                err.print(py);
                return;
            }
        };

        let readOptionString = |dict: &Bound<PyDict>, key: &str, label: &str| -> Option<String> {
            match dict.get_item(key) {
                Ok(Some(value)) => match value.extract::<Option<String>>() {
                    Ok(result) => result,
                    Err(err) => {
                        println!("Warning: {} {} {}", label, key, err);
                        None
                    }
                },
                Ok(None) => None,
                Err(err) => {
                    println!("Warning: {} {} {}", label, key, err);
                    None
                }
            }
        };

        let readOptionVecString = |dict: &Bound<PyDict>, key: &str, label: &str| -> Option<Vec<String>> {
            match dict.get_item(key) {
                Ok(Some(value)) => match value.extract::<Option<Vec<String>>>() {
                    Ok(result) => result,
                    Err(err) => {
                        println!("Warning: {} {} {}", label, key, err);
                        None
                    }
                },
                Ok(None) => None,
                Err(err) => {
                    println!("Warning: {} {} {}", label, key, err);
                    None
                }
            }
        };

        let readOptionVecF32 = |dict: &Bound<PyDict>, key: &str, label: &str| -> Option<Vec<f32>> {
            match dict.get_item(key) {
                Ok(Some(value)) => match value.extract::<Option<Vec<f32>>>() {
                    Ok(result) => result,
                    Err(err) => {
                        println!("Warning: {} {} {}", label, key, err);
                        None
                    }
                },
                Ok(None) => None,
                Err(err) => {
                    println!("Warning: {} {} {}", label, key, err);
                    None
                }
            }
        };

        let mut applyState = |stateDict: &Bound<PyDict>| {
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

        if let Ok(dict) = updateResult.downcast::<PyDict>() {
            applyState(dict);
            return;
        }

        if let Ok(getState) = apiModule.getattr("getState") {
            if let Ok(stateResult) = getState.call0() {
                if let Ok(dict) = stateResult.downcast::<PyDict>() {
                    applyState(dict);
                }
            }
        }
    });

    nextPhase
}
