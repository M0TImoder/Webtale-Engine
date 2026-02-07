use bevy::prelude::*;
use bevy::sprite::Anchor;
use rustpython_vm::builtins::PyDictRef;
use rustpython_vm::compiler::Mode;
use rustpython_vm::scope::Scope;
use std::collections::HashMap;
use crate::components::*;
use crate::constants::*;
use crate::python_scripts;
use crate::resources::*;
use crate::systems::phase;

pub fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    python_runtime: NonSend<PythonRuntime>,
    mut window_query: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.visible = false;
    }

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

    spawn_game_objects(&mut commands, &asset_server, &game_fonts, &python_runtime);

    commands.insert_resource(game_fonts);
}

pub fn spawn_game_objects(commands: &mut Commands, asset_server: &AssetServer, game_fonts: &GameFonts, python_runtime: &PythonRuntime) {
    let mut game_state = GameState {
        hp: 0.0,
        max_hp: 0.0,
        lv: 1,
        name: String::new(),
        
        speed: 0.0,
        attack: 0.0,
        defense: 0.0,
        invincibility_duration: 0.0,

        enemy_hp: 0,
        enemy_max_hp: 0,
        enemy_atk: 0,
        enemy_def: 0,
        enemy_name: String::new(),
        enemy_dialog_text: String::new(),
        enemy_act_commands: vec![],
        enemy_act_texts: HashMap::new(),
        enemy_bubble_messages: vec![],
        enemy_body_texture: String::new(),
        enemy_head_texture: String::new(),
        enemy_head_yoffset: 0.0,
        enemy_base_x: 0.0,
        enemy_base_y: 0.0,
        enemy_scale: 1.0,
        enemy_attacks: vec![],
        phase_name: String::new(),
        phase_turn: 0,
        turn_count: 0,
        enemy_bubble_texture: "texture/blcon/spr_blconsm.png".to_string(),
        enemy_bubble_message_override: None,
        enemy_bubble_pos_override: None,

        mn_fight: 0, 
        my_fight: 0,
        menu_layer: MENU_LAYER_TOP,
        menu_coords: vec![0; 11],

        inventory: vec![],
        equipped_items: vec![],
        item_page: 0,
        
        dialog_text: String::new(),
        
        bubble_timer: Timer::from_seconds(3.0, TimerMode::Once),
        damage_display_timer: Timer::from_seconds(1.0, TimerMode::Once),
        turn_timer: -1.0,
        invincibility_timer: 0.0,
        last_player_action: String::new(),
        last_act_command: None,
    };

    let project_name = PROJECT_NAME;
    let mut item_dictionary = ItemDictionary::default();
    let mut phase_script_name = String::new();

    python_runtime.interpreter.enter(|vm| {
        let run_script = |code: &str, filename: &str| -> Option<Scope> {
            let scope = vm.new_scope_with_builtins();
            let code_obj = match vm.compile(code, Mode::Exec, filename.to_string()) {
                Ok(code_obj) => code_obj,
                Err(err) => {
                    println!("Warning: python compile {} {:?}", filename, err);
                    return None;
                }
            };
            if let Err(err) = vm.run_code_obj(code_obj, scope.clone()) {
                vm.print_exception(err.clone());
                return None;
            }
            Some(scope)
        };

        let read_string = |dict: &PyDictRef, key: &str, label: &str| -> Option<String> {
            match dict.get_item_opt(key, vm) {
                Ok(Some(value)) => match value.try_into_value(vm) {
                    Ok(result) => Some(result),
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: {} {} {:?}", label, key, err);
                        None
                    }
                },
                Ok(None) => {
                    println!("Warning: {} missing {}", label, key);
                    None
                }
                Err(err) => {
                    vm.print_exception(err.clone());
                    println!("Warning: {} {} {:?}", label, key, err);
                    None
                }
            }
        };

        let read_f32 = |dict: &PyDictRef, key: &str, label: &str| -> Option<f32> {
            match dict.get_item_opt(key, vm) {
                Ok(Some(value)) => match value.try_into_value(vm) {
                    Ok(result) => Some(result),
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: {} {} {:?}", label, key, err);
                        None
                    }
                },
                Ok(None) => {
                    println!("Warning: {} missing {}", label, key);
                    None
                }
                Err(err) => {
                    vm.print_exception(err.clone());
                    println!("Warning: {} {} {:?}", label, key, err);
                    None
                }
            }
        };

        let read_i32 = |dict: &PyDictRef, key: &str, label: &str| -> Option<i32> {
            match dict.get_item_opt(key, vm) {
                Ok(Some(value)) => match value.try_into_value(vm) {
                    Ok(result) => Some(result),
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: {} {} {:?}", label, key, err);
                        None
                    }
                },
                Ok(None) => {
                    println!("Warning: {} missing {}", label, key);
                    None
                }
                Err(err) => {
                    vm.print_exception(err.clone());
                    println!("Warning: {} {} {:?}", label, key, err);
                    None
                }
            }
        };

        let read_vec_string = |dict: &PyDictRef, key: &str, label: &str| -> Option<Vec<String>> {
            match dict.get_item_opt(key, vm) {
                Ok(Some(value)) => match value.try_into_value(vm) {
                    Ok(result) => Some(result),
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: {} {} {:?}", label, key, err);
                        None
                    }
                },
                Ok(None) => {
                    println!("Warning: {} missing {}", label, key);
                    None
                }
                Err(err) => {
                    vm.print_exception(err.clone());
                    println!("Warning: {} {} {:?}", label, key, err);
                    None
                }
            }
        };

        let item_script = match python_scripts::get_item_script(project_name) {
            Some(script) => script,
            None => {
                println!("Warning: Could not load projects/{}/properties/item.py", project_name);
                ""
            }
        };
        if !item_script.is_empty() {
            if let Some(scope) = run_script(item_script, "item.py") {
                match scope.globals.get_item_opt("getItemData", vm) {
                    Ok(Some(func)) => match vm.invoke(&func, ()) {
                        Ok(result) => match result.try_into_value::<PyDictRef>(vm) {
                            Ok(dict) => {
                                for (key, value) in &dict {
                                    let item_name: String = match key.try_into_value(vm) {
                                        Ok(name) => name,
                                        Err(err) => {
                                            vm.print_exception(err.clone());
                                            println!("Warning: itemData key {:?}", err);
                                            continue;
                                        }
                                    };
                                    let data: PyDictRef = match value.try_into_value(vm) {
                                        Ok(data) => data,
                                        Err(err) => {
                                            vm.print_exception(err.clone());
                                            println!("Warning: itemData value {:?}", err);
                                            continue;
                                        }
                                    };
                                    let heal = read_i32(&data, "heal", "itemData").unwrap_or(0);
                                    let attack = read_i32(&data, "attack", "itemData").unwrap_or(0);
                                    let defense = read_i32(&data, "defense", "itemData").unwrap_or(0);
                                    let text = read_string(&data, "text", "itemData").unwrap_or_default();

                                    item_dictionary.0.insert(item_name, ItemInfo { heal_amount: heal, attack, defense, text });
                                }
                            }
                            Err(err) => {
                                vm.print_exception(err.clone());
                                println!("Warning: itemData result {:?}", err);
                            }
                        },
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: itemData call {:?}", err);
                        }
                    },
                    Ok(None) => println!("Warning: itemData missing getItemData"),
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: itemData lookup {:?}", err);
                    }
                }
            }
        }

        let player_status_script = match python_scripts::get_player_status_script(project_name) {
            Some(script) => script,
            None => {
                println!("Warning: Could not load projects/{}/properties/playerStatus.py", project_name);
                ""
            }
        };
        if !player_status_script.is_empty() {
            if let Some(scope) = run_script(player_status_script, "playerStatus.py") {
                match scope.globals.get_item_opt("getPlayerStatus", vm) {
                    Ok(Some(func)) => match vm.invoke(&func, ()) {
                        Ok(result) => match result.try_into_value::<PyDictRef>(vm) {
                            Ok(dict) => {
                                if let Some(name) = read_string(&dict, "name", "playerStatus") {
                                    game_state.name = name;
                                }
                                if let Some(lv) = read_i32(&dict, "lv", "playerStatus") {
                                    game_state.lv = lv;
                                }
                                if let Some(max_hp) = read_f32(&dict, "maxHp", "playerStatus") {
                                    game_state.max_hp = max_hp;
                                }
                                if let Some(hp) = read_f32(&dict, "hp", "playerStatus") {
                                    game_state.hp = hp;
                                }
                                if let Some(speed) = read_f32(&dict, "speed", "playerStatus") {
                                    game_state.speed = speed;
                                }
                                if let Some(attack) = read_f32(&dict, "attack", "playerStatus") {
                                    game_state.attack = attack;
                                }
                                if let Some(defense) = read_f32(&dict, "defense", "playerStatus") {
                                    game_state.defense = defense;
                                }
                                if let Some(inv_dur) = read_f32(&dict, "invincibilityDuration", "playerStatus") {
                                    game_state.invincibility_duration = inv_dur;
                                }
                                if let Some(inventory) = read_vec_string(&dict, "inventory", "playerStatus") {
                                    game_state.inventory = inventory;
                                }
                                if let Some(equipped_items) = read_vec_string(&dict, "equippedItems", "playerStatus") {
                                    game_state.equipped_items = equipped_items;
                                }
                            }
                            Err(err) => {
                                vm.print_exception(err.clone());
                                println!("Warning: playerStatus result {:?}", err);
                            }
                        },
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: playerStatus call {:?}", err);
                        }
                    },
                    Ok(None) => println!("Warning: playerStatus missing getPlayerStatus"),
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: playerStatus lookup {:?}", err);
                    }
                }
            }
        }

        let enemy_status_script = match python_scripts::get_enemy_status_script(project_name) {
            Some(script) => script,
            None => {
                println!("Warning: Could not load projects/{}/properties/enemyStatus.py", project_name);
                ""
            }
        };
        if !enemy_status_script.is_empty() {
            if let Some(scope) = run_script(enemy_status_script, "enemyStatus.py") {
                match scope.globals.get_item_opt("getEnemyStatus", vm) {
                    Ok(Some(func)) => match vm.invoke(&func, ()) {
                        Ok(result) => match result.try_into_value::<PyDictRef>(vm) {
                            Ok(dict) => {
                                if let Some(hp) = read_i32(&dict, "enemyHp", "enemyStatus") {
                                    game_state.enemy_hp = hp;
                                }
                                if let Some(max_hp) = read_i32(&dict, "enemyMaxHp", "enemyStatus") {
                                    game_state.enemy_max_hp = max_hp;
                                }
                                if let Some(atk) = read_i32(&dict, "enemyAtk", "enemyStatus") {
                                    game_state.enemy_atk = atk;
                                }
                                if let Some(def) = read_i32(&dict, "enemyDef", "enemyStatus") {
                                    game_state.enemy_def = def;
                                }
                                if let Some(name) = read_string(&dict, "enemyName", "enemyStatus") {
                                    game_state.enemy_name = name;
                                }
                                if let Some(dialog_text) = read_string(&dict, "dialogText", "enemyStatus") {
                                    game_state.enemy_dialog_text = dialog_text;
                                }
                                if let Some(phase_script) = read_string(&dict, "phaseScript", "enemyStatus") {
                                    phase_script_name = phase_script;
                                }
                                if let Some(attacks) = read_vec_string(&dict, "attackPatterns", "enemyStatus") {
                                    game_state.enemy_attacks = attacks;
                                }
                                if let Some(commands) = read_vec_string(&dict, "actCommands", "enemyStatus") {
                                    game_state.enemy_act_commands = commands;
                                }
                                match dict.get_item_opt("actTexts", vm) {
                                    Ok(Some(act_texts_obj)) => match act_texts_obj.try_into_value::<PyDictRef>(vm) {
                                        Ok(act_texts) => {
                                            for (key, value) in &act_texts {
                                                let command: String = match key.try_into_value(vm) {
                                                    Ok(name) => name,
                                                    Err(err) => {
                                                        vm.print_exception(err.clone());
                                                        println!("Warning: enemyStatus actTexts key {:?}", err);
                                                        continue;
                                                    }
                                                };
                                                let text: String = match value.try_into_value(vm) {
                                                    Ok(result) => result,
                                                    Err(err) => {
                                                        vm.print_exception(err.clone());
                                                        println!("Warning: enemyStatus actTexts value {:?}", err);
                                                        continue;
                                                    }
                                                };
                                                game_state.enemy_act_texts.insert(command, text);
                                            }
                                        }
                                        Err(err) => {
                                            vm.print_exception(err.clone());
                                            println!("Warning: enemyStatus actTexts {:?}", err);
                                        }
                                    },
                                    Ok(None) => println!("Warning: enemyStatus missing actTexts"),
                                    Err(err) => {
                                        vm.print_exception(err.clone());
                                        println!("Warning: enemyStatus actTexts {:?}", err);
                                    }
                                }
                                if let Some(messages) = read_vec_string(&dict, "bubbleMessages", "enemyStatus") {
                                    game_state.enemy_bubble_messages = messages;
                                }
                                if let Some(body_texture) = read_string(&dict, "bodyTexture", "enemyStatus") {
                                    game_state.enemy_body_texture = body_texture;
                                }
                                if let Some(head_texture) = read_string(&dict, "headTexture", "enemyStatus") {
                                    game_state.enemy_head_texture = head_texture;
                                }
                                if let Some(head_yoffset) = read_f32(&dict, "headYOffset", "enemyStatus") {
                                    game_state.enemy_head_yoffset = head_yoffset;
                                }
                                if let Some(base_x) = read_f32(&dict, "baseX", "enemyStatus") {
                                    game_state.enemy_base_x = base_x;
                                }
                                if let Some(base_y) = read_f32(&dict, "baseY", "enemyStatus") {
                                    game_state.enemy_base_y = base_y;
                                }
                                if let Some(scale) = read_f32(&dict, "scale", "enemyStatus") {
                                    game_state.enemy_scale = scale;
                                }
                            }
                            Err(err) => {
                                vm.print_exception(err.clone());
                                println!("Warning: enemyStatus result {:?}", err);
                            }
                        },
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: enemyStatus call {:?}", err);
                        }
                    },
                    Ok(None) => println!("Warning: enemyStatus missing getEnemyStatus"),
                    Err(err) => {
                        vm.print_exception(err.clone());
                        println!("Warning: enemyStatus lookup {:?}", err);
                    }
                }
            }
        }
    });

    if game_state.name.is_empty() {
        println!("Warning: playerStatus missing name");
    }

    if game_state.max_hp <= 0.0 {
        println!("Warning: playerStatus maxHp invalid");
        game_state.max_hp = 1.0;
    }

    if game_state.hp <= 0.0 {
        println!("Warning: playerStatus hp invalid");
        game_state.hp = game_state.max_hp;
    }

    if game_state.speed <= 0.0 {
        println!("Warning: playerStatus speed invalid");
    }

    if game_state.invincibility_duration <= 0.0 {
        println!("Warning: playerStatus invincibilityDuration invalid");
    }

    if game_state.enemy_max_hp <= 0 {
        println!("Warning: enemyMaxHp invalid");
        game_state.enemy_max_hp = 1;
    }

    if game_state.enemy_name.is_empty() {
        println!("Warning: enemyStatus missing enemyName");
    }

    if game_state.enemy_body_texture.is_empty() {
        println!("Warning: enemyStatus missing bodyTexture");
    }

    if game_state.enemy_head_texture.is_empty() {
        println!("Warning: enemyStatus missing headTexture");
    }

    game_state.phase_name = phase::resolve_initial_phase(project_name, &phase_script_name);
    if !game_state.phase_name.is_empty() {
        if let Some(next_phase) = phase::apply_phase_update(&mut game_state, project_name, "start", python_runtime) {
            if next_phase != game_state.phase_name {
                game_state.phase_name = next_phase;
                game_state.phase_turn = 0;
                let _ = phase::apply_phase_update(&mut game_state, project_name, "start", python_runtime);
            }
        }
    }

    if !game_state.enemy_dialog_text.is_empty() {
        game_state.dialog_text = game_state.enemy_dialog_text.clone();
    }

    let enemy_base_x = game_state.enemy_base_x; 
    let enemy_base_y = game_state.enemy_base_y; 
    let enemy_scale = if game_state.enemy_scale <= 0.0 {
        println!("Warning: enemyStatus scale invalid");
        1.0
    } else {
        game_state.enemy_scale
    }; 

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(&game_state.enemy_body_texture),
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
            commands: game_state.enemy_act_commands.clone(),
        },
        Cleanup,
    ));

    let head_yoffset = game_state.enemy_head_yoffset; 
    let head_pos = gml_to_bevy(enemy_base_x, enemy_base_y - head_yoffset);
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(&game_state.enemy_head_texture),
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
            texture: asset_server.load("texture/heart/spr_heart_0.png"), 
            sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(16.0, 16.0)), ..default() },
            transform: Transform::from_translation(gml_to_bevy(0.0, 0.0) + Vec3::new(0.0, 0.0, Z_SOUL)),
            ..default()
        },
        Soul,
        Cleanup,
    ));

    let buttons = [
        (BTN_FIGHT_X, "texture/button/spr_fightbt_0.png", "texture/button/spr_fightbt_1.png", 0),
        (BTN_ACT_X,   "texture/button/spr_actbt_center_0.png", "texture/button/spr_actbt_center_1.png", 1),
        (BTN_ITEM_X,  "texture/button/spr_itembt_0.png",  "texture/button/spr_itembt_1.png",  2),
        (BTN_MERCY_X, "texture/button/spr_sparebt_0.png", "texture/button/spr_sparebt_1.png", 3),
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
    let font_style = TextStyle { font: game_fonts.main.clone(), font_size: font_size, color: COLOR_UI_TEXT };

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(&game_state.name, font_style.clone()),
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
            text: Text::from_section(format!("LV {}", game_state.lv), font_style.clone()),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(lv_x, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
            ..default()
        },
        LvText,
        Cleanup,
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("HP", TextStyle { font: game_fonts.hp_label.clone(), font_size: 10.0, color: COLOR_UI_TEXT }),
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
            text: Text::from_section(format!("{:.0} / {:.0}", game_state.hp, game_state.max_hp), font_style),
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
            full_text: game_state.dialog_text.clone(), 
            visible_chars: 0, 
            timer: Timer::from_seconds(0.03, TimerMode::Repeating), 
            finished: false 
        },
        MainDialogText,
        Cleanup,
    ));

    commands.insert_resource(item_dictionary);
    commands.insert_resource(game_state);
}

pub fn camera_scaling_system(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut projection_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let Ok(window) = window_query.get_single() else { return };
    if !window.visible {
        return;
    }

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
