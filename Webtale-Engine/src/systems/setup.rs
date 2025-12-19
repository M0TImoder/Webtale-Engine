use bevy::prelude::*;
use bevy::sprite::Anchor;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::fs;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn setup(
    mut commands: Commands, 
    assetServer: Res<AssetServer>,
    _windowQuery: Query<Entity, With<bevy::window::PrimaryWindow>>,
) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));

    let fontMain = assetServer.load("font/Mars_Needs_Cunnilingus.ttf");
    let fontDialog = assetServer.load("font/8bitOperatorPlus-Bold.ttf");
    let fontHpLabel = assetServer.load("font/8-BIT_WO.ttf");
    let fontDamage = assetServer.load("font/hachicro.TTF");

    let gameFonts = GameFonts {
        main: fontMain.clone(),
        dialog: fontDialog.clone(),
        hpLabel: fontHpLabel.clone(),
        damage: fontDamage.clone(), 
    };

    spawnGameObjects(&mut commands, &assetServer, &gameFonts);

    commands.insert_resource(gameFonts);
}

pub fn spawnGameObjects(commands: &mut Commands, assetServer: &AssetServer, gameFonts: &GameFonts) {
    let mut gameState = GameState {
        hp: 20.0,
        maxHp: 20.0,
        lv: 1,
        name: "CHARA".to_string(),
        
        speed: 150.0,
        attack: 20.0,
        invincibilityDuration: 1.0,

        enemyHp: 30,
        enemyMaxHp: 30,
        enemyDef: 0,
        enemyAttacks: vec![],

        mnFight: 0, 
        myFight: 0,
        menuLayer: MENU_LAYER_TOP,
        menuCoords: vec![0; 11],

        inventory: vec![],
        itemPage: 0,
        
        dialogText: "* Froggit hops close!".to_string(),
        
        bubbleTimer: Timer::from_seconds(3.0, TimerMode::Once),
        damageDisplayTimer: Timer::from_seconds(1.0, TimerMode::Once),
        turnTimer: -1.0,
        invincibilityTimer: 0.0,
    };

    let projectName = PROJECT_NAME;

    let mut itemDictionary = ItemDictionary::default();
    let itemPath = format!("projects/{}/properties/item.py", projectName);

    if let Ok(script) = fs::read_to_string(&itemPath) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script, "item.py", "item") {
                if let Ok(func) = module.getattr("getItemData") {
                    if let Ok(result) = func.call0() {
                        if let Ok(dict) = result.downcast::<PyDict>() {
                            for (key, value) in dict.iter() {
                                let itemName: String = key.extract().unwrap_or_default();
                                if let Ok(data) = value.downcast::<PyDict>() {
                                    let heal: i32 = data.get_item("heal").ok().flatten().and_then(|v| v.extract().ok()).unwrap_or(0);
                                    let text: String = data.get_item("text").ok().flatten().and_then(|v| v.extract().ok()).unwrap_or_default();
                                    
                                    itemDictionary.0.insert(itemName, ItemInfo { healAmount: heal, text });
                                }
                            }
                        }
                    }
                }

                if let Ok(func) = module.getattr("getInitialInventory") {
                    if let Ok(result) = func.call0() {
                        if let Ok(list) = result.downcast::<PyList>() {
                             if let Ok(inv) = list.extract() {
                                 gameState.inventory = inv;
                             }
                        }
                    }
                }
            }
        });
    } else {
        println!("Warning: Could not load {}", itemPath);
    }

    let playerStatusPath = format!("projects/{}/properties/playerStatus.py", projectName);
    if let Ok(script) = fs::read_to_string(&playerStatusPath) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script, "playerStatus.py", "playerStatus") {
                if let Ok(func) = module.getattr("getPlayerStatus") {
                    if let Ok(result) = func.call0() {
                        if let Ok(dict) = result.downcast::<PyDict>() {
                            if let Some(name) = dict.get_item("name").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.name = name;
                            }
                            if let Some(lv) = dict.get_item("lv").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.lv = lv;
                            }
                            if let Some(maxHp) = dict.get_item("maxHp").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.maxHp = maxHp;
                            }
                            if let Some(hp) = dict.get_item("hp").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.hp = hp;
                            }
                            if let Some(speed) = dict.get_item("speed").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.speed = speed;
                            }
                            if let Some(attack) = dict.get_item("attack").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.attack = attack;
                            }
                            if let Some(invDur) = dict.get_item("invincibilityDuration").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.invincibilityDuration = invDur;
                            }
                        }
                    }
                }
            }
        });
    } else {
        println!("Warning: Could not load {}", playerStatusPath);
    }

    let enemyStatusPath = format!("projects/{}/properties/enemyStatus.py", projectName);
    if let Ok(script) = fs::read_to_string(&enemyStatusPath) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script, "enemyStatus.py", "enemyStatus") {
                if let Ok(func) = module.getattr("getEnemyStatus") {
                    if let Ok(result) = func.call0() {
                        if let Ok(dict) = result.downcast::<PyDict>() {
                            if let Some(hp) = dict.get_item("enemyHp").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.enemyHp = hp;
                            }
                            if let Some(maxHp) = dict.get_item("enemyMaxHp").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.enemyMaxHp = maxHp;
                            }
                            if let Some(def) = dict.get_item("enemyDef").ok().flatten().and_then(|v| v.extract().ok()) {
                                gameState.enemyDef = def;
                            }
                            if let Some(attacks) = dict.get_item("attackPatterns").ok().flatten().and_then(|v| v.extract::<Vec<String>>().ok()) {
                                gameState.enemyAttacks = attacks;
                            }
                        }
                    }
                }
            }
        });
    } else {
        println!("Warning: Could not load {}", enemyStatusPath);
    }

    commands.insert_resource(itemDictionary);
    commands.insert_resource(gameState);

    let enemyBaseX = 320.0; 
    let enemyBaseY = 160.0; 
    let enemyScale = 1.0; 

    commands.spawn((
        SpriteBundle {
            texture: assetServer.load("enemy/spr_froglegs_0.png"),
            sprite: Sprite { color: Color::WHITE, custom_size: None, ..default() },
            transform: Transform {
                translation: gml_to_bevy(enemyBaseX, enemyBaseY) + Vec3::new(0.0, 0.0, Z_ENEMY_BODY),
                scale: Vec3::splat(enemyScale), 
                ..default()
            },
            ..default()
        },
        EnemyBody, 
        ActCommands {
            commands: vec!["Check".to_string(), "Compliment".to_string(), "Threaten".to_string()],
        },
        Cleanup,
    ));

    let headYOffset = 22.0; 
    let headPos = gml_to_bevy(enemyBaseX, enemyBaseY - headYOffset);
    commands.spawn((
        SpriteBundle {
            texture: assetServer.load("enemy/spr_froghead_0.png"),
            sprite: Sprite { color: Color::WHITE, custom_size: None, ..default() },
            transform: Transform {
                translation: headPos + Vec3::new(0.0, 0.0, Z_ENEMY_HEAD),
                scale: Vec3::splat(enemyScale), 
                ..default()
            },
            ..default()
        },
        EnemyHead { baseY: headPos.y, timer: 0.0 },
        EnemyBody, 
        Cleanup,
    ));

    commands.spawn((
        SpriteBundle {
            texture: assetServer.load("heart/spr_heart_0.png"), 
            sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(16.0, 16.0)), ..default() },
            transform: Transform::from_translation(gml_to_bevy(0.0, 0.0) + Vec3::new(0.0, 0.0, Z_SOUL)),
            ..default()
        },
        Soul,
        Cleanup,
    ));

    let buttons = [
        (BTN_FIGHT_X, "button/spr_fightbt_0.png", "button/spr_fightbt_1.png", 0),
        (BTN_ACT_X,   "button/spr_actbt_center_0.png", "button/spr_actbt_center_1.png", 1),
        (BTN_ITEM_X,  "button/spr_itembt_0.png",  "button/spr_itembt_1.png",  2),
        (BTN_MERCY_X, "button/spr_sparebt_0.png", "button/spr_sparebt_1.png", 3),
    ];

    for (x, normalPath, selectedPath, idx) in buttons {
        let normalHandle = assetServer.load(normalPath);
        let selectedHandle = assetServer.load(selectedPath);

        commands.spawn((
            SpriteBundle {
                texture: normalHandle.clone(),
                sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(110.0, 42.0)), ..default() },
                transform: Transform::from_translation(gml_to_bevy(x + 55.0, BUTTON_Y_GML + 21.0) + Vec3::new(0.0, 0.0, Z_BUTTON)),
                ..default()
            },
            ButtonVisual { index: idx, normalTexture: normalHandle, selectedTexture: selectedHandle },
            Cleanup,
        ));
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: Color::WHITE, ..default() },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, Z_BORDER)),
            ..default()
        },
        BorderVisual,
        Cleanup,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: Color::BLACK, ..default() },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, Z_BG)),
            ..default()
        },
        BackgroundVisual,
        Cleanup,
    ));

    let fontSize = 23.0; 
    let fontStyle = TextStyle { font: gameFonts.main.clone(), font_size: fontSize, color: COLOR_UI_TEXT };

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("CHARA", fontStyle.clone()),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(30.0, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
            ..default()
        },
        PlayerNameText,
        Cleanup,
    ));

    let lvX = 30.0 + 85.0 + 15.0; 
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("LV 1", fontStyle.clone()),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(lvX, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
            ..default()
        },
        LvText,
        Cleanup,
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("HP", TextStyle { font: gameFonts.hpLabel.clone(), font_size: 9.0, color: COLOR_UI_TEXT }),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(225.0, 405.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
            ..default()
        },
        Cleanup,
    ));

    let hpBarX = 250.0;
    let hpBarY = 401.0;

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: COLOR_HP_RED, anchor: Anchor::TopLeft, ..default() },
            transform: Transform::from_translation(gml_to_bevy(hpBarX, hpBarY) + Vec3::new(0.0, 0.0, Z_HP_BAR_BG)),
            ..default()
        },
        HpBarRed,
        Cleanup,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: COLOR_HP_YELLOW, anchor: Anchor::TopLeft, ..default() },
            transform: Transform::from_translation(gml_to_bevy(hpBarX, hpBarY) + Vec3::new(0.0, 0.0, Z_HP_BAR_FG)),
            ..default()
        },
        HpBarYellow,
        Cleanup,
    ));

    let hpTextX = 250.0 + 24.0 + 15.0;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("20 / 20", fontStyle),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(hpTextX, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
            ..default()
        },
        HpText,
        Cleanup,
    ));
    
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("", TextStyle { font: gameFonts.dialog.clone(), font_size: 32.0, color: Color::WHITE }),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
            ..default()
        },
        Typewriter { 
            fullText: "* Froggit hops close!".to_string(), 
            visibleChars: 0, 
            timer: Timer::from_seconds(0.03, TimerMode::Repeating), 
            finished: false 
        },
        MainDialogText,
        Cleanup,
    ));
}

pub fn cameraScalingSystem(
    windowQuery: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut projectionQuery: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    if let Ok(window) = windowQuery.get_single() {
        if let Ok(mut projection) = projectionQuery.get_single_mut() {
            let targetRatio = 640.0 / 480.0;
            let windowRatio = window.width() / window.height();

            if windowRatio > targetRatio {
                projection.scaling_mode = bevy::render::camera::ScalingMode::FixedVertical(480.0);
            } else {
                projection.scaling_mode = bevy::render::camera::ScalingMode::FixedHorizontal(640.0);
            }
        }
    }
}
