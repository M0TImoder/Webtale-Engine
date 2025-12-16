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
    // --- ここから修正: Pythonから初期設定を読み込む ---
    let project_name = PROJECT_NAME; // constants.rsから取得
    let properties_path = format!("projects/{}/properties/properties.py", project_name);

    // デフォルトのGameState（読み込み失敗時のフォールバック用）
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

    // Pythonスクリプトの読み込みと適用
    if let Ok(script_content) = fs::read_to_string(&properties_path) {
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::from_code_bound(py, &script_content, "properties.py", "properties") {
                if let Ok(func) = module.getattr("get_initial_properties") {
                    if let Ok(result) = func.call0() {
                        // 結果を辞書型として取得
                        if let Ok(props) = result.downcast::<PyDict>() {
                            
                            // 修正: unwrap_or(game_state.xxx) をやめ、取得成功時のみ代入する形に変更
                            
                            if let Ok(Some(val)) = props.get_item("name") { 
                                if let Ok(v) = val.extract() { game_state.name = v; }
                            }
                            if let Ok(Some(val)) = props.get_item("lv") { 
                                if let Ok(v) = val.extract() { game_state.lv = v; }
                            }
                            if let Ok(Some(val)) = props.get_item("max_hp") { 
                                if let Ok(v) = val.extract() { game_state.max_hp = v; }
                            }
                            if let Ok(Some(val)) = props.get_item("hp") { 
                                if let Ok(v) = val.extract() { game_state.hp = v; }
                            }
                            
                            if let Ok(Some(val)) = props.get_item("speed") { 
                                if let Ok(v) = val.extract() { game_state.speed = v; }
                            }
                            if let Ok(Some(val)) = props.get_item("attack") { 
                                if let Ok(v) = val.extract() { game_state.attack = v; }
                            }
                            if let Ok(Some(val)) = props.get_item("invincibility_duration") { 
                                if let Ok(v) = val.extract() { game_state.invincibility_duration = v; }
                            }

                            if let Ok(Some(val)) = props.get_item("enemy_hp") { 
                                if let Ok(v) = val.extract() { game_state.enemy_hp = v; }
                            }
                            if let Ok(Some(val)) = props.get_item("enemy_max_hp") { 
                                if let Ok(v) = val.extract() { game_state.enemy_max_hp = v; }
                            }
                            if let Ok(Some(val)) = props.get_item("enemy_def") { 
                                if let Ok(v) = val.extract() { game_state.enemy_def = v; }
                            }

                            if let Ok(Some(val)) = props.get_item("inventory") { 
                                if let Ok(list) = val.downcast::<PyList>() {
                                    if let Ok(v) = list.extract() { game_state.inventory = v; }
                                }
                            }
                        }
                    } else {
                        eprintln!("Failed to call get_initial_properties in properties.py");
                    }
                } else {
                    eprintln!("Function get_initial_properties not found in properties.py");
                }
            } else {
                eprintln!("Failed to load properties.py module");
            }
        });
    } else {
        eprintln!("Could not read properties.py at {}", properties_path);
    }

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
