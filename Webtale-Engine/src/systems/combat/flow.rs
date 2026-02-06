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

pub fn battleFlowControl(
    mut commands: Commands,
    mut gameState: ResMut<GameState>,
    assetServer: Res<AssetServer>,
    gameFonts: Res<GameFonts>,
    python_runtime: NonSend<PythonRuntime>,
    _time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>, 
    mut boxRes: ResMut<BattleBox>,
    bubbles: Query<Entity, With<SpeechBubble>>,
    bubbleTextQuery: Query<&Typewriter, With<SpeechBubble>>, 
    mut soulQuery: Query<&mut Transform, With<Soul>>,
    mut eguiContexts: EguiContexts,
    editorQuery: Query<Entity, With<EditorWindow>>,
) {
    if let Ok(editorEntity) = editorQuery.get_single() {
        if eguiContexts.ctx_for_window_mut(editorEntity).wants_keyboard_input() {
            return;
        }
    }

    if gameState.mnFight == 1 {
        if bubbles.is_empty() {
            gameState.turnCount += 1;
            gameState.phaseTurn += 1;
            if let Some(nextPhase) = phase::applyPhaseUpdate(&mut gameState, PROJECT_NAME, "turn", &python_runtime) {
                if nextPhase != gameState.phaseName {
                    gameState.phaseName = nextPhase;
                    gameState.phaseTurn = 1;
                    let _ = phase::applyPhaseUpdate(&mut gameState, PROJECT_NAME, "turn", &python_runtime);
                }
            }

            boxRes.target = Rect::new(32.0, 250.0, 602.0, 385.0);
            let bubblePos = gameState.enemyBubblePosOverride.unwrap_or(Vec2::new(320.0 + 40.0, 160.0 - 95.0));
            let bubbleX = bubblePos.x; 
            let bubbleY = bubblePos.y; 
            let bubbleTexture = if gameState.enemyBubbleTexture.is_empty() {
                "texture/blcon/spr_blconsm.png".to_string()
            } else {
                gameState.enemyBubbleTexture.clone()
            };
            commands.spawn((
                SpriteBundle {
                    texture: assetServer.load(bubbleTexture), 
                    sprite: Sprite { 
                        color: Color::WHITE, 
                        custom_size: Some(Vec2::new(100.0, 80.0)), 
                        anchor: Anchor::TopLeft, 
                        ..default() 
                    },
                    transform: Transform::from_translation(gml_to_bevy(bubbleX, bubbleY) + Vec3::new(0.0, 0.0, Z_BUBBLE)),
                    ..default()
                },
                SpeechBubble,
                Cleanup,
            ));
            let msg = if let Some(message) = gameState.enemyBubbleMessageOverride.take() {
                message
            } else if gameState.enemyBubbleMessages.is_empty() {
                println!("Warning: enemy bubble messages missing");
                "...".to_string()
            } else {
                let idx = rand::thread_rng().gen_range(0..gameState.enemyBubbleMessages.len());
                gameState.enemyBubbleMessages[idx].clone()
            };
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("", TextStyle { font: gameFonts.dialog.clone(), font_size: 24.0, color: Color::BLACK }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(bubbleX + 15.0, bubbleY + 15.0) + Vec3::new(0.0, 0.0, Z_BUBBLE_TEXT)),
                    ..default()
                },
                Typewriter { fullText: msg, visibleChars: 0, timer: Timer::from_seconds(0.05, TimerMode::Repeating), finished: false },
                SpeechBubble, 
                Cleanup,
            ));
        }
        
        let mut isFinished = false;
        if let Ok(writer) = bubbleTextQuery.get_single() {
            if writer.finished {
                isFinished = true;
            }
        }

        if isFinished && input.just_pressed(KeyCode::KeyZ) {
            for entity in bubbles.iter() { commands.entity(entity).despawn_recursive(); }
            
            gameState.mnFight = 2; 
            gameState.turnTimer = -1.0; 
            
            boxRes.target = Rect::new(217.0, 125.0, 417.0, 385.0);
            
            let boxCenterX = (217.0 + 417.0) / 2.0;
            let boxCenterY = (125.0 + 385.0) / 2.0;
            if let Ok(mut t) = soulQuery.get_single_mut() {
                t.translation = gml_to_bevy(boxCenterX, boxCenterY) + Vec3::new(0.0, 0.0, Z_SOUL);
            }
        }
    }
}

pub fn combatTurnManager(
    mut commands: Commands,
    assetServer: Res<AssetServer>,
    time: Res<Time>,
    mut gameState: ResMut<GameState>,
    mut battleBox: ResMut<BattleBox>,
    python_runtime: NonSend<PythonRuntime>,
    bulletQuery: Query<Entity, With<PythonBullet>>,
    mut scripts: ResMut<DanmakuScripts>,
) {
    if gameState.mnFight == 2 {
        if gameState.turnTimer < 0.0 {
            gameState.turnTimer = 5.0; 
            
            let attackPatterns = &gameState.enemyAttacks;
            let scriptName = if !attackPatterns.is_empty() {
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..attackPatterns.len());
                attackPatterns[idx].clone()
            } else {
                println!("Warning: enemyStatus attackPatterns missing");
                "frogJump".to_string() 
            };
            
            let scriptContent = match python_scripts::get_danmaku_script(PROJECT_NAME, &scriptName) {
                Some(content) => content,
                None => {
                    println!("Warning: script missing projects/{}/danmaku/{}.py", PROJECT_NAME, scriptName);
                    return;
                }
            };

            let apiContent = match python_scripts::get_danmaku_api_script(PROJECT_NAME) {
                Some(content) => content,
                None => {
                    println!("Warning: script missing projects/{}/danmaku/api.py", PROJECT_NAME);
                    return;
                }
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

                let apiModule = match run_module(apiContent, "api.py", "api") {
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
                if let Err(err) = modules.set_item("api", apiModule.clone(), vm) {
                    vm.print_exception(err.clone());
                    return;
                }

                let module = match run_module(scriptContent, &format!("{}.py", scriptName), &scriptName) {
                    Some(module) => module,
                    None => return,
                };

                let initFunc = match module.get_attr("init", vm) {
                    Ok(func) => func,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let initResult = match vm.invoke(&initFunc, ()) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let initData: PyDictRef = match initResult.try_into_value(vm) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: danmaku init {:?}", err);
                        return;
                    }
                };

                let boxDataObj = match initData.get_item_opt("box", vm) {
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
                let _boxData: Vec<f32> = match boxDataObj.try_into_value(vm) {
                    Ok(value) => value,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };

                let texturePathObj = match initData.get_item_opt("textureWait", vm) {
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
                let texturePath: String = match texturePathObj.try_into_value(vm) {
                    Ok(value) => value,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };

                let spawnX = ORIGIN_X + battleBox.current.max.x - 40.0;
                let spawnY = ORIGIN_Y - battleBox.current.max.y + 40.0;

                let spawnFunc = match module.get_attr("spawn", vm) {
                    Ok(func) => func,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };
                let bulletObj: PyObjectRef = match vm.invoke(&spawnFunc, ()) {
                    Ok(result) => result,
                    Err(err) => {
                        vm.print_exception(err.clone());
                        return;
                    }
                };

                match bulletObj.get_attr("setPos", vm) {
                    Ok(setPos) => {
                        if let Err(err) = vm.invoke(&setPos, (spawnX, spawnY)) {
                            vm.print_exception(err.clone());
                        }
                    }
                    Err(err) => {
                        vm.print_exception(err.clone());
                    }
                }

                let damage = match bulletObj.get_attr("damage", vm) {
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
                        texture: assetServer.load(texturePath),
                        transform: Transform::from_xyz(spawnX, spawnY, 30.0).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    PythonBullet {
                        scriptName: scriptName.clone(),
                        bulletData: bulletObj.clone(),
                        damage,
                    },
                    Cleanup,
                ));

                scripts.modules.insert("api".to_string(), apiModule);
                scripts.modules.insert(scriptName, module);
            });
        }

        gameState.turnTimer -= time.delta_seconds();

        if gameState.turnTimer <= 0.0 {
            for entity in bulletQuery.iter() {
                commands.entity(entity).despawn();
            }
            
            gameState.mnFight = 3;
            gameState.turnTimer = -1.0;
        }
    } else if gameState.mnFight == 3 {
        gameState.mnFight = 0;
        gameState.myFight = 0;
        gameState.menuLayer = 0;
        gameState.dialogText = gameState.enemyDialogText.clone(); 
        
        battleBox.target = Rect::new(32.0, 250.0, 602.0, 385.0);
    }
}
