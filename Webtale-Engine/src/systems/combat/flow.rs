use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;
use bevy_egui::EguiContexts;
use pyo3::prelude::*;
use pyo3::types::PyDict; 
use crate::components::*;
use crate::resources::*;
use crate::constants::*;
use crate::systems::phase;

pub fn battleFlowControl(
    mut commands: Commands,
    mut gameState: ResMut<GameState>,
    assetServer: Res<AssetServer>,
    gameFonts: Res<GameFonts>,
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
            if let Some(nextPhase) = phase::applyPhaseUpdate(&mut gameState, PROJECT_NAME, "turn") {
                if nextPhase != gameState.phaseName {
                    gameState.phaseName = nextPhase;
                    gameState.phaseTurn = 1;
                    let _ = phase::applyPhaseUpdate(&mut gameState, PROJECT_NAME, "turn");
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
            
            let relativePath = format!("projects/{}/danmaku", PROJECT_NAME);
            let scriptFilePath = format!("{}/{}.py", relativePath, scriptName);

            let scriptContent = match std::fs::read_to_string(&scriptFilePath) {
                Ok(content) => content,
                Err(err) => {
                    println!("Warning: script load {} {}", scriptFilePath, err);
                    String::new()
                }
            };
            
            Python::with_gil(|py| {
                let sys = PyModule::import_bound(py, "sys").expect("Failed to import sys");
                let path = sys.getattr("path").expect("Failed to get sys.path");
                
                let envPath = std::env::current_dir().unwrap().join(&relativePath);
                let _ = path.call_method1("append", (envPath.to_str().unwrap(),));

                let apiPath = format!("{}/api.py", relativePath);
                let apiContent = match std::fs::read_to_string(&apiPath) {
                    Ok(content) => content,
                    Err(err) => {
                        println!("Warning: script load {} {}", apiPath, err);
                        String::new()
                    }
                };

                if apiContent.is_empty() {
                    return;
                }

                let apiModule = match PyModule::from_code_bound(py, &apiContent, "api.py", "api") {
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
                if let Err(err) = modules.set_item("api", &apiModule) {
                    err.print(py);
                    return;
                }

                if scriptContent.is_empty() {
                    return;
                }

                let module = match PyModule::from_code_bound(py, &scriptContent, &format!("{}.py", scriptName), &scriptName) {
                    Ok(module) => module,
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };
                
                let initFunc = match module.getattr("init") {
                    Ok(func) => func,
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };
                let initResult = match initFunc.call0() {
                    Ok(result) => result,
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };
                let initData: &Bound<PyDict> = match initResult.downcast() {
                    Ok(result) => result,
                    Err(err) => {
                        println!("Warning: danmaku init {}", err);
                        return;
                    }
                };
                
                let boxDataObj = match initData.get_item("box") {
                    Ok(Some(value)) => value,
                    Ok(None) => {
                        println!("Warning: danmaku box missing");
                        return;
                    }
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };
                let _boxData: Vec<f32> = match boxDataObj.extract() {
                    Ok(value) => value,
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };

                let texturePathObj = match initData.get_item("textureWait") {
                    Ok(Some(value)) => value,
                    Ok(None) => {
                        println!("Warning: danmaku textureWait missing");
                        return;
                    }
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };
                let texturePath: String = match texturePathObj.extract() {
                    Ok(value) => value,
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };
                
                let spawnX = ORIGIN_X + battleBox.current.max.x - 40.0;
                let spawnY = ORIGIN_Y - battleBox.current.max.y + 40.0;

                let spawnFunc = match module.getattr("spawn") {
                    Ok(func) => func,
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };
                let bulletObj: PyObject = match spawnFunc.call0() {
                    Ok(result) => result.into(),
                    Err(err) => {
                        err.print(py);
                        return;
                    }
                };
                
                let bulletBound = bulletObj.bind(py);
                if let Err(err) = bulletBound.call_method1("setPos", (spawnX, spawnY)) {
                    err.print(py);
                }

                let damage = match bulletBound.getattr("damage") {
                    Ok(value) => match value.extract::<i32>() {
                        Ok(result) => result,
                        Err(err) => {
                            println!("Warning: bullet damage {}", err);
                            0
                        }
                    },
                    Err(err) => {
                        println!("Warning: bullet damage {}", err);
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
                        bulletData: bulletObj,
                        damage,
                    },
                    Cleanup,
                ));
                
                scripts.modules.insert("api".to_string(), apiModule.into());
                scripts.modules.insert(scriptName, module.into());
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
