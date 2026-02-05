use bevy::prelude::*;
use bevy::sprite::Anchor;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::fs;
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

pub fn setup(
    mut commands: Commands, 
    assetServer: Res<AssetServer>,
    mut windowQuery: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    if let Ok(mut window) = windowQuery.get_single_mut() {
        window.visible = false;
    }

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
        hp: 0.0,
        maxHp: 0.0,
        lv: 1,
        name: String::new(),
        
        speed: 0.0,
        attack: 0.0,
        defense: 0.0,
        invincibilityDuration: 0.0,

        enemyHp: 0,
        enemyMaxHp: 0,
        enemyAtk: 0,
        enemyDef: 0,
        enemyName: String::new(),
        enemyDialogText: String::new(),
        enemyActCommands: vec![],
        enemyActTexts: HashMap::new(),
        enemyBubbleMessages: vec![],
        enemyBodyTexture: String::new(),
        enemyHeadTexture: String::new(),
        enemyHeadYOffset: 0.0,
        enemyBaseX: 0.0,
        enemyBaseY: 0.0,
        enemyScale: 1.0,
        enemyAttacks: vec![],

        mnFight: 0, 
        myFight: 0,
        menuLayer: MENU_LAYER_TOP,
        menuCoords: vec![0; 11],

        inventory: vec![],
        equippedItems: vec![],
        itemPage: 0,
        
        dialogText: String::new(),
        
        bubbleTimer: Timer::from_seconds(3.0, TimerMode::Once),
        damageDisplayTimer: Timer::from_seconds(1.0, TimerMode::Once),
        turnTimer: -1.0,
        invincibilityTimer: 0.0,
    };

    let projectName = PROJECT_NAME;

    let readString = |dict: &Bound<PyDict>, key: &str, label: &str| -> Option<String> {
        match dict.get_item(key) {
            Ok(Some(value)) => match value.extract::<String>() {
                Ok(result) => Some(result),
                Err(err) => {
                    println!("Warning: {} {} {}", label, key, err);
                    None
                }
            },
            Ok(None) => {
                println!("Warning: {} missing {}", label, key);
                None
            }
            Err(err) => {
                println!("Warning: {} {} {}", label, key, err);
                None
            }
        }
    };

    let readF32 = |dict: &Bound<PyDict>, key: &str, label: &str| -> Option<f32> {
        match dict.get_item(key) {
            Ok(Some(value)) => match value.extract::<f32>() {
                Ok(result) => Some(result),
                Err(err) => {
                    println!("Warning: {} {} {}", label, key, err);
                    None
                }
            },
            Ok(None) => {
                println!("Warning: {} missing {}", label, key);
                None
            }
            Err(err) => {
                println!("Warning: {} {} {}", label, key, err);
                None
            }
        }
    };

    let readI32 = |dict: &Bound<PyDict>, key: &str, label: &str| -> Option<i32> {
        match dict.get_item(key) {
            Ok(Some(value)) => match value.extract::<i32>() {
                Ok(result) => Some(result),
                Err(err) => {
                    println!("Warning: {} {} {}", label, key, err);
                    None
                }
            },
            Ok(None) => {
                println!("Warning: {} missing {}", label, key);
                None
            }
            Err(err) => {
                println!("Warning: {} {} {}", label, key, err);
                None
            }
        }
    };

    let readVecString = |dict: &Bound<PyDict>, key: &str, label: &str| -> Option<Vec<String>> {
        match dict.get_item(key) {
            Ok(Some(value)) => match value.extract::<Vec<String>>() {
                Ok(result) => Some(result),
                Err(err) => {
                    println!("Warning: {} {} {}", label, key, err);
                    None
                }
            },
            Ok(None) => {
                println!("Warning: {} missing {}", label, key);
                None
            }
            Err(err) => {
                println!("Warning: {} {} {}", label, key, err);
                None
            }
        }
    };

    let mut itemDictionary = ItemDictionary::default();
    let itemPath = format!("projects/{}/properties/item.wep", projectName);

    if let Ok(script) = fs::read_to_string(&itemPath) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script, "item.wep", "item") {
                if let Ok(func) = module.getattr("getItemData") {
                    if let Ok(result) = func.call0() {
                        if let Ok(dict) = result.downcast::<PyDict>() {
                            for (key, value) in dict.iter() {
                                let itemName: String = match key.extract() {
                                    Ok(name) => name,
                                    Err(err) => {
                                        println!("Warning: itemData key {}", err);
                                        continue;
                                    }
                                };
                                if let Ok(data) = value.downcast::<PyDict>() {
                                    let heal = readI32(data, "heal", "itemData").unwrap_or(0);
                                    let attack = readI32(data, "attack", "itemData").unwrap_or(0);
                                    let defense = readI32(data, "defense", "itemData").unwrap_or(0);
                                    let text = readString(data, "text", "itemData").unwrap_or_default();
                                    
                                    itemDictionary.0.insert(itemName, ItemInfo { healAmount: heal, attack, defense, text });
                                }
                            }
                        }
                    }
                }
            }
        });
    } else {
        println!("Warning: Could not load {}", itemPath);
    }

    let playerStatusPath = format!("projects/{}/properties/playerStatus.wep", projectName);
    if let Ok(script) = fs::read_to_string(&playerStatusPath) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script, "playerStatus.wep", "playerStatus") {
                if let Ok(func) = module.getattr("getPlayerStatus") {
                    if let Ok(result) = func.call0() {
                        if let Ok(dict) = result.downcast::<PyDict>() {
                            if let Some(name) = readString(dict, "name", "playerStatus") {
                                gameState.name = name;
                            }
                            if let Some(lv) = readI32(dict, "lv", "playerStatus") {
                                gameState.lv = lv;
                            }
                            if let Some(maxHp) = readF32(dict, "maxHp", "playerStatus") {
                                gameState.maxHp = maxHp;
                            }
                            if let Some(hp) = readF32(dict, "hp", "playerStatus") {
                                gameState.hp = hp;
                            }
                            if let Some(speed) = readF32(dict, "speed", "playerStatus") {
                                gameState.speed = speed;
                            }
                            if let Some(attack) = readF32(dict, "attack", "playerStatus") {
                                gameState.attack = attack;
                            }
                            if let Some(defense) = readF32(dict, "defense", "playerStatus") {
                                gameState.defense = defense;
                            }
                            if let Some(invDur) = readF32(dict, "invincibilityDuration", "playerStatus") {
                                gameState.invincibilityDuration = invDur;
                            }
                            if let Some(inventory) = readVecString(dict, "inventory", "playerStatus") {
                                gameState.inventory = inventory;
                            }
                            if let Some(equippedItems) = readVecString(dict, "equippedItems", "playerStatus") {
                                gameState.equippedItems = equippedItems;
                            }
                        }
                    }
                }
            }
        });
    } else {
        println!("Warning: Could not load {}", playerStatusPath);
    }

    if gameState.name.is_empty() {
        println!("Warning: playerStatus missing name");
    }

    if gameState.maxHp <= 0.0 {
        println!("Warning: playerStatus maxHp invalid");
        gameState.maxHp = 1.0;
    }

    if gameState.hp <= 0.0 {
        println!("Warning: playerStatus hp invalid");
        gameState.hp = gameState.maxHp;
    }

    if gameState.speed <= 0.0 {
        println!("Warning: playerStatus speed invalid");
    }

    if gameState.invincibilityDuration <= 0.0 {
        println!("Warning: playerStatus invincibilityDuration invalid");
    }

    let enemyStatusPath = format!("projects/{}/properties/enemyStatus.wep", projectName);
    if let Ok(script) = fs::read_to_string(&enemyStatusPath) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script, "enemyStatus.wep", "enemyStatus") {
                if let Ok(func) = module.getattr("getEnemyStatus") {
                    if let Ok(result) = func.call0() {
                        if let Ok(dict) = result.downcast::<PyDict>() {
                            if let Some(hp) = readI32(dict, "enemyHp", "enemyStatus") {
                                gameState.enemyHp = hp;
                            }
                            if let Some(maxHp) = readI32(dict, "enemyMaxHp", "enemyStatus") {
                                gameState.enemyMaxHp = maxHp;
                            }
                            if let Some(atk) = readI32(dict, "enemyAtk", "enemyStatus") {
                                gameState.enemyAtk = atk;
                            }
                            if let Some(def) = readI32(dict, "enemyDef", "enemyStatus") {
                                gameState.enemyDef = def;
                            }
                            if let Some(name) = readString(dict, "enemyName", "enemyStatus") {
                                gameState.enemyName = name;
                            }
                            if let Some(dialogText) = readString(dict, "dialogText", "enemyStatus") {
                                gameState.enemyDialogText = dialogText;
                            }
                            if let Some(attacks) = readVecString(dict, "attackPatterns", "enemyStatus") {
                                gameState.enemyAttacks = attacks;
                            }
                            if let Some(commands) = readVecString(dict, "actCommands", "enemyStatus") {
                                gameState.enemyActCommands = commands;
                            }
                            if let Ok(Some(actTextsObj)) = dict.get_item("actTexts") {
                                if let Ok(actTexts) = actTextsObj.downcast::<PyDict>() {
                                    for (key, value) in actTexts.iter() {
                                        let command: String = match key.extract() {
                                            Ok(name) => name,
                                            Err(err) => {
                                                println!("Warning: enemyStatus actTexts key {}", err);
                                                continue;
                                            }
                                        };
                                        let text: String = match value.extract() {
                                            Ok(result) => result,
                                            Err(err) => {
                                                println!("Warning: enemyStatus actTexts value {}", err);
                                                continue;
                                            }
                                        };
                                        gameState.enemyActTexts.insert(command, text);
                                    }
                                } else {
                                    println!("Warning: enemyStatus actTexts not dict");
                                }
                            } else {
                                println!("Warning: enemyStatus missing actTexts");
                            }
                            if let Some(messages) = readVecString(dict, "bubbleMessages", "enemyStatus") {
                                gameState.enemyBubbleMessages = messages;
                            }
                            if let Some(bodyTexture) = readString(dict, "bodyTexture", "enemyStatus") {
                                gameState.enemyBodyTexture = bodyTexture;
                            }
                            if let Some(headTexture) = readString(dict, "headTexture", "enemyStatus") {
                                gameState.enemyHeadTexture = headTexture;
                            }
                            if let Some(headYOffset) = readF32(dict, "headYOffset", "enemyStatus") {
                                gameState.enemyHeadYOffset = headYOffset;
                            }
                            if let Some(baseX) = readF32(dict, "baseX", "enemyStatus") {
                                gameState.enemyBaseX = baseX;
                            }
                            if let Some(baseY) = readF32(dict, "baseY", "enemyStatus") {
                                gameState.enemyBaseY = baseY;
                            }
                            if let Some(scale) = readF32(dict, "scale", "enemyStatus") {
                                gameState.enemyScale = scale;
                            }
                        }
                    }
                }
            }
        });
    } else {
        println!("Warning: Could not load {}", enemyStatusPath);
    }

    if gameState.enemyMaxHp <= 0 {
        println!("Warning: enemyMaxHp invalid");
        gameState.enemyMaxHp = 1;
    }

    if gameState.enemyName.is_empty() {
        println!("Warning: enemyStatus missing enemyName");
    }

    if gameState.enemyBodyTexture.is_empty() {
        println!("Warning: enemyStatus missing bodyTexture");
    }

    if gameState.enemyHeadTexture.is_empty() {
        println!("Warning: enemyStatus missing headTexture");
    }

    if !gameState.enemyDialogText.is_empty() {
        gameState.dialogText = gameState.enemyDialogText.clone();
    }

    let enemyBaseX = gameState.enemyBaseX; 
    let enemyBaseY = gameState.enemyBaseY; 
    let enemyScale = if gameState.enemyScale <= 0.0 {
        println!("Warning: enemyStatus scale invalid");
        1.0
    } else {
        gameState.enemyScale
    }; 

    commands.spawn((
        SpriteBundle {
            texture: assetServer.load(&gameState.enemyBodyTexture),
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
            commands: gameState.enemyActCommands.clone(),
        },
        Cleanup,
    ));

    let headYOffset = gameState.enemyHeadYOffset; 
    let headPos = gml_to_bevy(enemyBaseX, enemyBaseY - headYOffset);
    commands.spawn((
        SpriteBundle {
            texture: assetServer.load(&gameState.enemyHeadTexture),
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
            text: Text::from_section(&gameState.name, fontStyle.clone()),
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
            text: Text::from_section(format!("LV {}", gameState.lv), fontStyle.clone()),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(lvX, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
            ..default()
        },
        LvText,
        Cleanup,
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("HP", TextStyle { font: gameFonts.hpLabel.clone(), font_size: 10.0, color: COLOR_UI_TEXT }),
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
            text: Text::from_section(format!("{:.0} / {:.0}", gameState.hp, gameState.maxHp), fontStyle),
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
            fullText: gameState.dialogText.clone(), 
            visibleChars: 0, 
            timer: Timer::from_seconds(0.03, TimerMode::Repeating), 
            finished: false 
        },
        MainDialogText,
        Cleanup,
    ));

    commands.insert_resource(itemDictionary);
    commands.insert_resource(gameState);
}

pub fn cameraScalingSystem(
    windowQuery: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut projectionQuery: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let Ok(window) = windowQuery.get_single() else { return };
    if !window.visible {
        return;
    }

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
