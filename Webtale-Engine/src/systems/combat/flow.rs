use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;
use bevy_egui::EguiContexts;
use pyo3::prelude::*;
use pyo3::types::PyDict; 
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

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
            boxRes.target = Rect::new(32.0, 250.0, 602.0, 385.0);
            let bubbleX = 320.0 + 40.0; 
            let bubbleY = 160.0 - 95.0; 
            commands.spawn((
                SpriteBundle {
                    texture: assetServer.load("blcon/spr_blconsm.png"), 
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
            let messages = ["Ribbit, ribbit.", "Croak.", "Hop, hop."];
            let msg = messages[rand::thread_rng().gen_range(0..messages.len())];
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("", TextStyle { font: gameFonts.dialog.clone(), font_size: 24.0, color: Color::BLACK }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(bubbleX + 15.0, bubbleY + 15.0) + Vec3::new(0.0, 0.0, Z_BUBBLE_TEXT)),
                    ..default()
                },
                Typewriter { fullText: msg.to_string(), visibleChars: 0, timer: Timer::from_seconds(0.05, TimerMode::Repeating), finished: false },
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
                "frogJump".to_string() 
            };
            
            let relativePath = format!("projects/{}/danmaku", PROJECT_NAME);
            let scriptFilePath = format!("{}/{}.py", relativePath, scriptName);

            let scriptContent = std::fs::read_to_string(&scriptFilePath).unwrap_or_default();
            
            Python::with_gil(|py| {
                let sys = PyModule::import_bound(py, "sys").expect("Failed to import sys");
                let path = sys.getattr("path").expect("Failed to get sys.path");
                
                let envPath = std::env::current_dir().unwrap().join(relativePath);
                let _ = path.call_method1("append", (envPath.to_str().unwrap(),));

                let module = PyModule::from_code_bound(py, &scriptContent, &format!("{}.py", scriptName), &scriptName).expect("Failed to load python script");
                
                let initFunc = module.getattr("init").expect("Failed to get init function");
                let initResult = initFunc.call0().expect("Failed to call init");
                let initData: &Bound<PyDict> = initResult.downcast().expect("init should return dict");
                
                let boxDataObj = initData.get_item("box").expect("box missing").expect("box None");
                let _boxData: Vec<f32> = boxDataObj.extract().expect("box format error");

                let texturePathObj = initData.get_item("textureWait").expect("textureWait missing").expect("textureWait None");
                let texturePath: String = texturePathObj.extract().unwrap();
                
                let spawnX = ORIGIN_X + battleBox.current.max.x - 40.0;
                let spawnY = ORIGIN_Y - battleBox.current.max.y + 40.0;

                let spawnFunc = module.getattr("spawn").expect("Failed to get spawn function");
                let bulletObj: PyObject = spawnFunc.call0().expect("Failed to call spawn").into();
                
                let bulletBound = bulletObj.bind(py);
                let _ = bulletBound.call_method1("setPos", (spawnX, spawnY));

                commands.spawn((
                    SpriteBundle {
                        texture: assetServer.load(texturePath),
                        transform: Transform::from_xyz(spawnX, spawnY, 30.0).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    PythonBullet {
                        scriptName: scriptName.clone(),
                        bulletData: bulletObj,
                        damage: 4,
                    },
                    Cleanup,
                ));
                
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
        gameState.dialogText = "* Froggit hops close!".to_string(); 
        
        battleBox.target = Rect::new(32.0, 250.0, 602.0, 385.0);
    }
}
