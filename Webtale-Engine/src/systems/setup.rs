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
    asset_server: Res<AssetServer>,
    _window_query: Query<Entity, With<bevy::window::PrimaryWindow>>,
) {

    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));

    let font_main = asset_server.load("font/Mars_Needs_Cunnilingus.ttf");
    let font_dialog = asset_server.load("font/8bitOperatorPlus-Bold.ttf");
    let font_hp_label = asset_server.load("font/8-BIT_WO.ttf");
    let font_damage = asset_server.load("font/hachicro.TTF");

    let game_fonts = GameFonts {
        main: font_main.clone(),
        dialog: font_dialog.clone(),
        hp_label: font_hp_label.clone(),
        damage: font_damage.clone(), 
    };

    spawn_game_objects(&mut commands, &asset_server, &game_fonts);

    commands.insert_resource(game_fonts);
}

pub fn spawn_game_objects(commands: &mut Commands, asset_server: &AssetServer, game_fonts: &GameFonts) {
    let mut game_state = GameState {
        hp: 20.0,
        max_hp: 20.0,
        lv: 1,
        name: "CHARA".to_string(),
        
        speed: 150.0,
        attack: 20.0,
        invincibility_duration: 1.0,

        enemy_hp: 30,
        enemy_max_hp: 30,
        enemy_def: 0,
        enemy_attacks: vec![],

        mnfight: 0, 
        myfight: 0,
        menu_layer: MENU_LAYER_TOP,
        menu_coords: vec![0; 11],

        inventory: vec![],
        item_page: 0,
        
        dialog_text: "* Froggit hops close!".to_string(),
        
        bubble_timer: Timer::from_seconds(3.0, TimerMode::Once),
        damage_display_timer: Timer::from_seconds(1.0, TimerMode::Once),
        turntimer: -1.0,
        invincibility_timer: 0.0,
    };

    let project_name = PROJECT_NAME;

    let mut item_dictionary = ItemDictionary::default();
    let item_path = format!("projects/{}/properties/item.py", project_name);

    if let Ok(script) = fs::read_to_string(&item_path) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script, "item.py", "item") {
                if let Ok(func) = module.getattr("get_item_data") {
                    if let Ok(result) = func.call0() {
                        if let Ok(dict) = result.downcast::<PyDict>() {
                            for (key, value) in dict.iter() {
                                let item_name: String = key.extract().unwrap_or_default();
                                if let Ok(data) = value.downcast::<PyDict>() {
                                    let heal: i32 = data.get_item("heal").ok().flatten().and_then(|v| v.extract().ok()).unwrap_or(0);
                                    let text: String = data.get_item("text").ok().flatten().and_then(|v| v.extract().ok()).unwrap_or_default();
                                    
                                    item_dictionary.0.insert(item_name, ItemInfo { heal_amount: heal, text });
                                }
                            }
                        }
                    }
                }

                if let Ok(func) = module.getattr("get_initial_inventory") {
                    if let Ok(result) = func.call0() {
                        if let Ok(list) = result.downcast::<PyList>() {
                             if let Ok(inv) = list.extract() {
                                 game_state.inventory = inv;
                             }
                        }
                    }
                }
            }
        });
    } else {
        println!("Warning: Could not load {}", item_path);
    }

    let enemy_status_path = format!("projects/{}/properties/enemyStatus.py", project_name);
    if let Ok(script) = fs::read_to_string(&enemy_status_path) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script, "enemyStatus.py", "enemyStatus") {
                if let Ok(func) = module.getattr("get_enemy_status") {
                    if let Ok(result) = func.call0() {
                        if let Ok(dict) = result.downcast::<PyDict>() {
                            if let Some(hp) = dict.get_item("enemy_hp").ok().flatten().and_then(|v| v.extract().ok()) {
                                game_state.enemy_hp = hp;
                            }
                            if let Some(max_hp) = dict.get_item("enemy_max_hp").ok().flatten().and_then(|v| v.extract().ok()) {
                                game_state.enemy_max_hp = max_hp;
                            }
                            if let Some(def) = dict.get_item("enemy_def").ok().flatten().and_then(|v| v.extract().ok()) {
                                game_state.enemy_def = def;
                            }
                            if let Some(attacks) = dict.get_item("attack_patterns").ok().flatten().and_then(|v| v.extract::<Vec<String>>().ok()) {
                                game_state.enemy_attacks = attacks;
                            }
                        }
                    }
                }
            }
        });
    } else {
        println!("Warning: Could not load {}", enemy_status_path);
    }

    commands.insert_resource(item_dictionary);
    commands.insert_resource(game_state);

    let enemy_base_x = 320.0; 
    let enemy_base_y = 160.0; 
    let enemy_scale = 1.0; 

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("enemy/spr_froglegs_0.png"),
            sprite: Sprite { color: Color::WHITE, custom_size: None, ..default() },
            transform: Transform {
                translation: gml_to_bevy(enemy_base_x, enemy_base_y) + Vec3::new(0.0, 0.0, Z_ENEMY_BODY),
                scale: Vec3::splat(enemy_scale), 
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

    let head_y_offset = 22.0; 
    let head_pos = gml_to_bevy(enemy_base_x, enemy_base_y - head_y_offset);
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("enemy/spr_froghead_0.png"),
            sprite: Sprite { color: Color::WHITE, custom_size: None, ..default() },
            transform: Transform {
                translation: head_pos + Vec3::new(0.0, 0.0, Z_ENEMY_HEAD),
                scale: Vec3::splat(enemy_scale), 
                ..default()
            },
            ..default()
        },
        EnemyHead { base_y: head_pos.y, timer: 0.0 },
        EnemyBody, 
        Cleanup,
    ));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("heart/spr_heart_0.png"), 
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

    for (x, normal_path, selected_path, idx) in buttons {
        let normal_handle = asset_server.load(normal_path);
        let selected_handle = asset_server.load(selected_path);

        commands.spawn((
            SpriteBundle {
                texture: normal_handle.clone(),
                sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(110.0, 42.0)), ..default() },
                transform: Transform::from_translation(gml_to_bevy(x + 55.0, BUTTON_Y_GML + 21.0) + Vec3::new(0.0, 0.0, Z_BUTTON)),
                ..default()
            },
            ButtonVisual { index: idx, normal_texture: normal_handle, selected_texture: selected_handle },
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

    let font_size = 23.0; 
    let font_style = TextStyle { font: game_fonts.main.clone(), font_size, color: COLOR_UI_TEXT };

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("CHARA", font_style.clone()),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(30.0, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
            ..default()
        },
        PlayerNameText,
        Cleanup,
    ));

    let lv_x = 30.0 + 85.0 + 15.0; 
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("LV 1", font_style.clone()),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(lv_x, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
            ..default()
        },
        LvText,
        Cleanup,
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("HP", TextStyle { font: game_fonts.hp_label.clone(), font_size: 9.0, color: COLOR_UI_TEXT }),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(225.0, 405.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
            ..default()
        },
        Cleanup,
    ));

    let hp_bar_x = 250.0;
    let hp_bar_y = 401.0;

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: COLOR_HP_RED, anchor: Anchor::TopLeft, ..default() },
            transform: Transform::from_translation(gml_to_bevy(hp_bar_x, hp_bar_y) + Vec3::new(0.0, 0.0, Z_HP_BAR_BG)),
            ..default()
        },
        HpBarRed,
        Cleanup,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: COLOR_HP_YELLOW, anchor: Anchor::TopLeft, ..default() },
            transform: Transform::from_translation(gml_to_bevy(hp_bar_x, hp_bar_y) + Vec3::new(0.0, 0.0, Z_HP_BAR_FG)),
            ..default()
        },
        HpBarYellow,
        Cleanup,
    ));

    let hp_text_x = 250.0 + 24.0 + 15.0;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("20 / 20", font_style),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(hp_text_x, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
            ..default()
        },
        HpText,
        Cleanup,
    ));
    
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("", TextStyle { font: game_fonts.dialog.clone(), font_size: 32.0, color: Color::WHITE }),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
            ..default()
        },
        Typewriter { 
            full_text: "* Froggit hops close!".to_string(), 
            visible_chars: 0, 
            timer: Timer::from_seconds(0.03, TimerMode::Repeating), 
            finished: false 
        },
        MainDialogText,
        Cleanup,
    ));
}

// 復活させた関数
pub fn camera_scaling_system(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut projection_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    if let Ok(window) = window_query.get_single() {
        if let Ok(mut projection) = projection_query.get_single_mut() {
            let target_ratio = 640.0 / 480.0;
            let window_ratio = window.width() / window.height();

            if window_ratio > target_ratio {
                projection.scaling_mode = bevy::render::camera::ScalingMode::FixedVertical(480.0);
            } else {
                projection.scaling_mode = bevy::render::camera::ScalingMode::FixedHorizontal(640.0);
            }
        }
    }
}
