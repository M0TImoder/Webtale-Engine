use bevy::prelude::*;
use bevy::sprite::Anchor;
use rustpython_vm::builtins::PyDictRef;
use rustpython_vm::compiler::Mode;
use rustpython_vm::scope::Scope;
use rustpython_vm::PyObjectRef;
use std::collections::HashMap;
use crate::components::*;
use crate::constants::*;
use crate::python_scripts;
use crate::python_utils::{read_option_f32, read_option_i32, read_option_string, read_option_vec_string};
use crate::resources::*;
use crate::systems::phase;

// 初期セットアップ
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

// ゲームオブジェクト生成
pub fn spawn_game_objects(commands: &mut Commands, asset_server: &AssetServer, game_fonts: &GameFonts, python_runtime: &PythonRuntime) {
    let mut player_state = default_player_state();
    let mut enemy_state = default_enemy_state();
    let mut menu_state = default_menu_state();
    let mut combat_state = default_combat_state();

    let project_name = PROJECT_NAME;
    let mut item_dictionary = ItemDictionary::default();
    let mut phase_script_name = String::new();

    load_python_game_data(
        python_runtime,
        project_name,
        &mut player_state,
        &mut enemy_state,
        &mut item_dictionary,
        &mut phase_script_name,
    );

    validate_loaded_states(&mut player_state, &mut enemy_state);

    apply_initial_phase(
        project_name,
        &phase_script_name,
        python_runtime,
        &mut enemy_state,
        &mut combat_state,
        &mut menu_state,
    );

    if !enemy_state.dialog_text.is_empty() {
        menu_state.dialog_text = enemy_state.dialog_text.clone();
    }

    spawn_enemy_entities(commands, asset_server, &enemy_state);
    spawn_soul(commands, asset_server);
    spawn_menu_buttons(commands, asset_server);
    spawn_battle_box_visuals(commands);
    spawn_ui(commands, game_fonts, &player_state, &menu_state);

    commands.insert_resource(item_dictionary);
    commands.insert_resource(player_state);
    commands.insert_resource(enemy_state);
    commands.insert_resource(menu_state);
    commands.insert_resource(combat_state);
}

// プレイヤーデフォルト
fn default_player_state() -> PlayerState {
    PlayerState {
        hp: 0.0,
        max_hp: 0.0,
        lv: 1,
        name: String::new(),
        speed: 0.0,
        attack: 0.0,
        defense: 0.0,
        invincibility_duration: 0.0,
        invincibility_timer: 0.0,
        inventory: vec![],
        equipped_items: vec![],
    }
}

// 敵デフォルト
fn default_enemy_state() -> EnemyState {
    EnemyState {
        hp: 0,
        max_hp: 0,
        atk: 0,
        def: 0,
        name: String::new(),
        dialog_text: String::new(),
        act_commands: vec![],
        act_texts: HashMap::new(),
        bubble_messages: vec![],
        body_texture: String::new(),
        head_texture: String::new(),
        head_yoffset: 0.0,
        tachie_script: String::new(),
        head_sway_speed: 2.0,
        head_sway_amplitude: 2.0,
        base_x: 0.0,
        base_y: 0.0,
        scale: 1.0,
        attacks: vec![],
        bubble_texture: "texture/blcon/spr_blconsm.png".to_string(),
        bubble_message_override: None,
        bubble_pos_override: None,
    }
}

// メニューデフォルト
fn default_menu_state() -> MenuState {
    MenuState {
        menu_layer: MENU_LAYER_TOP,
        menu_coords: vec![0; 11],
        item_page: 0,
        dialog_text: String::new(),
    }
}

// 戦闘デフォルト
fn default_combat_state() -> CombatState {
    CombatState {
        mn_fight: MainFightState::Menu,
        my_fight: MessageFightState::None,
        phase_name: String::new(),
        phase_turn: 0,
        turn_count: 0,
        turn_timer: -1.0,
        bubble_timer: Timer::from_seconds(3.0, TimerMode::Once),
        damage_display_timer: Timer::from_seconds(1.0, TimerMode::Once),
        last_player_action: String::new(),
        last_act_command: None,
    }
}

// Pythonデータ読み込み
fn load_python_game_data(
    python_runtime: &PythonRuntime,
    project_name: &str,
    player_state: &mut PlayerState,
    enemy_state: &mut EnemyState,
    item_dictionary: &mut ItemDictionary,
    phase_script_name: &mut String,
) {
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

        let properties_script = match python_scripts::get_properties_script(project_name) {
            Some(script) => script,
            None => {
                println!("Warning: Could not load projects/{}/properties/properties.py", project_name);
                String::new()
            }
        };
        if !properties_script.is_empty() {
            if let Some(scope) = run_script(&properties_script, "properties.py") {
                let mut properties_class: Option<PyObjectRef> = None;
                for (_, value) in &scope.globals {
                    let flag = match value.get_attr("__is_properties__", vm) {
                        Ok(attr) => match attr.try_into_value::<bool>(vm) {
                            Ok(result) => result,
                            Err(_) => false,
                        },
                        Err(_) => false,
                    };
                    if flag {
                        properties_class = Some(value.clone());
                        break;
                    }
                }
                if properties_class.is_none() {
                    match scope.globals.get_item_opt("GameData", vm) {
                        Ok(Some(value)) => properties_class = Some(value),
                        Ok(None) => println!("Warning: properties missing GameData"),
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: properties lookup {:?}", err);
                        }
                    }
                }
                if let Some(class_obj) = properties_class {
                    let read_i32_any = |dict: &PyDictRef, keys: &[&str]| -> Option<i32> {
                        for key in keys {
                            if let Some(value) = read_option_i32(vm, dict, key, "properties", false) {
                                return Some(value);
                            }
                        }
                        None
                    };
                    let read_f32_any = |dict: &PyDictRef, keys: &[&str]| -> Option<f32> {
                        for key in keys {
                            if let Some(value) = read_option_f32(vm, dict, key, "properties", false) {
                                return Some(value);
                            }
                        }
                        None
                    };

                    match class_obj.get_attr("items", vm) {
                        Ok(items_obj) => match items_obj.try_into_value::<PyDictRef>(vm) {
                            Ok(items_dict) => {
                                for (key, value) in &items_dict {
                                    let item_name: String = match key.try_into_value(vm) {
                                        Ok(name) => name,
                                        Err(err) => {
                                            vm.print_exception(err.clone());
                                            println!("Warning: properties item key {:?}", err);
                                            continue;
                                        }
                                    };
                                    let data: PyDictRef = match value.try_into_value(vm) {
                                        Ok(data) => data,
                                        Err(err) => {
                                            vm.print_exception(err.clone());
                                            println!("Warning: properties item value {:?}", err);
                                            continue;
                                        }
                                    };
                                    let heal = read_option_i32(vm, &data, "heal", "properties", false).unwrap_or(0);
                                    let attack = read_option_i32(vm, &data, "attack", "properties", false).unwrap_or(0);
                                    let defense = read_option_i32(vm, &data, "defense", "properties", false).unwrap_or(0);
                                    let text = read_option_string(vm, &data, "text", "properties", false).unwrap_or_default();
                                    item_dictionary.0.insert(item_name, ItemInfo { heal_amount: heal, attack, defense, text });
                                }
                            }
                            Err(err) => {
                                vm.print_exception(err.clone());
                                println!("Warning: properties items invalid");
                            }
                        },
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: properties items missing");
                        }
                    }

                    match class_obj.get_attr("inventory", vm) {
                        Ok(inv_obj) => match inv_obj.try_into_value::<PyDictRef>(vm) {
                            Ok(inv_dict) => {
                                if let Some(items) = read_option_vec_string(vm, &inv_dict, "items", "properties", false) {
                                    player_state.inventory = items;
                                }
                                match inv_dict.get_item_opt("equipment", vm) {
                                    Ok(Some(equip_obj)) => match equip_obj.try_into_value::<PyDictRef>(vm) {
                                        Ok(equip_dict) => {
                                            if let Some(weapon) = read_option_string(vm, &equip_dict, "weapon", "properties", false) {
                                                if !weapon.is_empty() {
                                                    player_state.equipped_items.push(weapon);
                                                }
                                            }
                                            if let Some(armor) = read_option_string(vm, &equip_dict, "armor", "properties", false) {
                                                if !armor.is_empty() {
                                                    player_state.equipped_items.push(armor);
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            vm.print_exception(err.clone());
                                            println!("Warning: properties equipment invalid");
                                        }
                                    },
                                    Ok(None) => {}
                                    Err(err) => {
                                        vm.print_exception(err.clone());
                                        println!("Warning: properties equipment lookup {:?}", err);
                                    }
                                }
                            }
                            Err(err) => {
                                vm.print_exception(err.clone());
                                println!("Warning: properties inventory invalid");
                            }
                        },
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: properties inventory missing");
                        }
                    }

                    match class_obj.get_attr("status", vm) {
                        Ok(status_obj) => match status_obj.try_into_value::<PyDictRef>(vm) {
                            Ok(status_dict) => {
                                if let Some(name) = read_option_string(vm, &status_dict, "name", "properties", false) {
                                    player_state.name = name;
                                }
                                if let Some(lv) = read_i32_any(&status_dict, &["lv", "currentLevel"]) {
                                    player_state.lv = lv;
                                }
                                if let Some(max_hp) = read_f32_any(&status_dict, &["maxHp", "maxHP"]) {
                                    player_state.max_hp = max_hp;
                                }
                                if let Some(hp) = read_f32_any(&status_dict, &["hp", "currentHP"]) {
                                    player_state.hp = hp;
                                }
                                if let Some(speed) = read_f32_any(&status_dict, &["speed"]) {
                                    player_state.speed = speed;
                                }
                                if let Some(attack) = read_f32_any(&status_dict, &["attack"]) {
                                    player_state.attack = attack;
                                }
                                if let Some(defense) = read_f32_any(&status_dict, &["defense"]) {
                                    player_state.defense = defense;
                                }
                                if let Some(inv_dur) = read_f32_any(&status_dict, &["invincibilityDuration"]) {
                                    player_state.invincibility_duration = inv_dur;
                                }
                            }
                            Err(err) => {
                                vm.print_exception(err.clone());
                                println!("Warning: properties status invalid");
                            }
                        },
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: properties status missing");
                        }
                    }
                }
            }
        }

        let enemy_status_script = match python_scripts::get_enemy_status_script(project_name) {
            Some(script) => script,
            None => {
                println!("Warning: Could not load projects/{}/properties/enemyStatus.py", project_name);
                String::new()
            }
        };
        if !enemy_status_script.is_empty() {
            if let Some(scope) = run_script(&enemy_status_script, "enemyStatus.py") {
                match scope.globals.get_item_opt("getEnemyStatus", vm) {
                    Ok(Some(func)) => match vm.invoke(&func, ()) {
                        Ok(result) => match result.try_into_value::<PyDictRef>(vm) {
                            Ok(dict) => {
                                if let Some(hp) = read_option_i32(vm, &dict, "enemyHp", "enemyStatus", true) {
                                    enemy_state.hp = hp;
                                }
                                if let Some(max_hp) = read_option_i32(vm, &dict, "enemyMaxHp", "enemyStatus", true) {
                                    enemy_state.max_hp = max_hp;
                                }
                                if let Some(atk) = read_option_i32(vm, &dict, "enemyAtk", "enemyStatus", true) {
                                    enemy_state.atk = atk;
                                }
                                if let Some(def) = read_option_i32(vm, &dict, "enemyDef", "enemyStatus", true) {
                                    enemy_state.def = def;
                                }
                                if let Some(name) = read_option_string(vm, &dict, "enemyName", "enemyStatus", true) {
                                    enemy_state.name = name;
                                }
                                if let Some(dialog_text) = read_option_string(vm, &dict, "dialogText", "enemyStatus", true) {
                                    enemy_state.dialog_text = dialog_text;
                                }
                                if let Some(phase_script) = read_option_string(vm, &dict, "phaseScript", "enemyStatus", true) {
                                    *phase_script_name = phase_script;
                                }
                                if let Some(attacks) = read_option_vec_string(vm, &dict, "attackPatterns", "enemyStatus", true) {
                                    enemy_state.attacks = attacks;
                                }
                                if let Some(commands) = read_option_vec_string(vm, &dict, "actCommands", "enemyStatus", true) {
                                    enemy_state.act_commands = commands;
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
                                                enemy_state.act_texts.insert(command, text);
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
                                if let Some(messages) = read_option_vec_string(vm, &dict, "bubbleMessages", "enemyStatus", true) {
                                    enemy_state.bubble_messages = messages;
                                }
                                if let Some(body_texture) = read_option_string(vm, &dict, "bodyTexture", "enemyStatus", true) {
                                    enemy_state.body_texture = body_texture;
                                }
                                if let Some(head_texture) = read_option_string(vm, &dict, "headTexture", "enemyStatus", true) {
                                    enemy_state.head_texture = head_texture;
                                }
                                if let Some(head_yoffset) = read_option_f32(vm, &dict, "headYOffset", "enemyStatus", true) {
                                    enemy_state.head_yoffset = head_yoffset;
                                }
                                if let Some(tachie_script) = read_option_string(vm, &dict, "tachieScript", "enemyStatus", true) {
                                    enemy_state.tachie_script = tachie_script;
                                }
                                if let Some(base_x) = read_option_f32(vm, &dict, "baseX", "enemyStatus", true) {
                                    enemy_state.base_x = base_x;
                                }
                                if let Some(base_y) = read_option_f32(vm, &dict, "baseY", "enemyStatus", true) {
                                    enemy_state.base_y = base_y;
                                }
                                if let Some(scale) = read_option_f32(vm, &dict, "scale", "enemyStatus", true) {
                                    enemy_state.scale = scale;
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

        if !enemy_state.tachie_script.is_empty() {
            let tachie_script = match python_scripts::get_tachie_script(project_name, &enemy_state.tachie_script) {
                Some(script) => script,
                None => {
                    println!(
                        "Warning: Could not load projects/{}/tachie/{}.py",
                        project_name, enemy_state.tachie_script
                    );
                    String::new()
                }
            };
            if !tachie_script.is_empty() {
                let filename = format!("tachie/{}.py", enemy_state.tachie_script);
                if let Some(scope) = run_script(&tachie_script, &filename) {
                    match scope.globals.get_item_opt("getTachieData", vm) {
                        Ok(Some(func)) => match vm.invoke(&func, ()) {
                            Ok(result) => match result.try_into_value::<PyDictRef>(vm) {
                                Ok(dict) => {
                                    if let Some(speed) = read_option_f32(vm, &dict, "headSwaySpeed", "tachie", true) {
                                        enemy_state.head_sway_speed = speed;
                                    }
                                    if let Some(amplitude) = read_option_f32(vm, &dict, "headSwayAmplitude", "tachie", true) {
                                        enemy_state.head_sway_amplitude = amplitude;
                                    }
                                }
                                Err(err) => {
                                    vm.print_exception(err.clone());
                                    println!("Warning: tachie result {:?}", err);
                                }
                            },
                            Err(err) => {
                                vm.print_exception(err.clone());
                                println!("Warning: tachie call {:?}", err);
                            }
                        },
                        Ok(None) => println!("Warning: tachie missing getTachieData"),
                        Err(err) => {
                            vm.print_exception(err.clone());
                            println!("Warning: tachie lookup {:?}", err);
                        }
                    }
                }
            }
        }
    });
}

// 読み込み検証
fn validate_loaded_states(player_state: &mut PlayerState, enemy_state: &mut EnemyState) {
    if player_state.name.is_empty() {
        println!("Warning: playerStatus missing name");
    }

    if player_state.max_hp <= 0.0 {
        println!("Warning: playerStatus maxHp invalid");
        player_state.max_hp = 1.0;
    }

    if player_state.hp <= 0.0 {
        println!("Warning: playerStatus hp invalid");
        player_state.hp = player_state.max_hp;
    }

    if player_state.speed <= 0.0 {
        println!("Warning: playerStatus speed invalid");
    }

    if player_state.invincibility_duration <= 0.0 {
        println!("Warning: playerStatus invincibilityDuration invalid");
    }

    if enemy_state.max_hp <= 0 {
        println!("Warning: enemyMaxHp invalid");
        enemy_state.max_hp = 1;
    }

    if enemy_state.name.is_empty() {
        println!("Warning: enemyStatus missing enemyName");
    }

    if enemy_state.body_texture.is_empty() {
        println!("Warning: enemyStatus missing bodyTexture");
    }

    if enemy_state.head_texture.is_empty() {
        println!("Warning: enemyStatus missing headTexture");
    }
}

// 初期フェーズ適用
fn apply_initial_phase(
    project_name: &str,
    phase_script_name: &str,
    python_runtime: &PythonRuntime,
    enemy_state: &mut EnemyState,
    combat_state: &mut CombatState,
    menu_state: &mut MenuState,
) {
    combat_state.phase_name = phase::resolve_initial_phase(project_name, phase_script_name, python_runtime);
    if !combat_state.phase_name.is_empty() {
        if let Some(next_phase) = phase::apply_phase_update(enemy_state, combat_state, menu_state, project_name, "start", python_runtime) {
            if next_phase != combat_state.phase_name {
                combat_state.phase_name = next_phase;
                combat_state.phase_turn = 0;
                let _ = phase::apply_phase_update(enemy_state, combat_state, menu_state, project_name, "start", python_runtime);
            }
        }
    }
}

// 敵生成
fn spawn_enemy_entities(commands: &mut Commands, asset_server: &AssetServer, enemy_state: &EnemyState) {
    let enemy_base_x = enemy_state.base_x;
    let enemy_base_y = enemy_state.base_y;
    let enemy_scale = if enemy_state.scale <= 0.0 {
        println!("Warning: enemyStatus scale invalid");
        1.0
    } else {
        enemy_state.scale
    };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { image: asset_server.load(&enemy_state.body_texture), color: Color::WHITE, custom_size: None, ..default() },
            transform: Transform {
                translation: gml_to_bevy(enemy_base_x, enemy_base_y) + Vec3::new(0.0, 0.0, Z_ENEMY_BODY),
                scale: Vec3::splat(enemy_scale),
                ..default()
            },
            ..default()
        },
        EnemyBody,
        ActCommands {
            commands: enemy_state.act_commands.clone(),
        },
        Cleanup,
    ));

    let head_yoffset = enemy_state.head_yoffset;
    let head_pos = gml_to_bevy(enemy_base_x, enemy_base_y - head_yoffset);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { image: asset_server.load(&enemy_state.head_texture), color: Color::WHITE, custom_size: None, ..default() },
            transform: Transform {
                translation: head_pos + Vec3::new(0.0, 0.0, Z_ENEMY_HEAD),
                scale: Vec3::splat(enemy_scale),
                ..default()
            },
            ..default()
        },
        EnemyHead {
            base_y: head_pos.y,
            timer: 0.0,
            sway_speed: enemy_state.head_sway_speed,
            sway_amplitude: enemy_state.head_sway_amplitude,
        },
        EnemyBody,
        Cleanup,
    ));
}

// ソウル生成
fn spawn_soul(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { image: asset_server.load("texture/heart/spr_heart_0.png"), color: Color::WHITE, custom_size: Some(Vec2::new(16.0, 16.0)), ..default() },
            transform: Transform::from_translation(gml_to_bevy(0.0, 0.0) + Vec3::new(0.0, 0.0, Z_SOUL)),
            visibility: Visibility::Hidden,
            ..default()
        },
        Soul,
        Cleanup,
    ));
}

// メニューボタン生成
fn spawn_menu_buttons(commands: &mut Commands, asset_server: &AssetServer) {
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
                sprite: Sprite { image: normal_handle.clone(), color: Color::WHITE, custom_size: Some(Vec2::new(110.0, 42.0)), ..default() },
                transform: Transform::from_translation(gml_to_bevy(x + 55.0, BUTTON_Y_GML + 21.0) + Vec3::new(0.0, 0.0, Z_BUTTON)),
                ..default()
            },
            ButtonVisual { index: idx, normal_texture: normal_handle, selected_texture: selected_handle },
            Cleanup,
        ));
    }
}

// バトルボックス表示
fn spawn_battle_box_visuals(commands: &mut Commands) {
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
}

// UI生成
fn spawn_ui(commands: &mut Commands, game_fonts: &GameFonts, player_state: &PlayerState, menu_state: &MenuState) {
    let font_size = 32.0 * TEXT_SCALE;
    let font_style = TextFont { font: game_fonts.main.clone(), font_size, ..default() };
    let font_color = TextColor(COLOR_UI_TEXT);

    commands.spawn((
        Text2d::new(player_state.name.clone()),
        font_style.clone(),
        font_color,
        Anchor::TopLeft,
        Transform::from_translation(gml_to_bevy(30.0, 398.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
        PlayerNameText,
        Cleanup,
    ));

    let lv_x = 30.0 + 85.0 + 15.0;
    commands.spawn((
        Text2d::new(format!("LV {}", player_state.lv)),
        font_style.clone(),
        font_color,
        Anchor::TopLeft,
        Transform::from_translation(gml_to_bevy(lv_x, 398.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
        LvText,
        Cleanup,
    ));

    commands.spawn((
        Text2d::new("HP"),
        TextFont { font: game_fonts.hp_label.clone(), font_size: 14.0 * TEXT_SCALE, ..default() },
        TextColor(COLOR_UI_TEXT),
        Anchor::TopLeft,
        Transform::from_translation(gml_to_bevy(225.0, 405.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
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
        Text2d::new(format!("{:.0} / {:.0}", player_state.hp, player_state.max_hp)),
        font_style,
        font_color,
        Anchor::TopLeft,
        Transform::from_translation(gml_to_bevy(hp_text_x, 398.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
        HpText,
        Cleanup,
    ));

    commands.spawn((
        Text2d::new(""),
        TextFont { font: game_fonts.dialog.clone(), font_size: 32.0 * TEXT_SCALE, ..default() },
        TextColor(Color::WHITE),
        Anchor::TopLeft,
        Transform::from_translation(gml_to_bevy(52.0, 270.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
        Typewriter {
            full_text: menu_state.dialog_text.clone(),
            visible_chars: 0,
            timer: Timer::from_seconds(0.03, TimerMode::Repeating),
            finished: false
        },
        MainDialogText,
        Cleanup,
    ));
}

// カメラスケール調整
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
            projection.scaling_mode = bevy::render::camera::ScalingMode::FixedVertical { viewport_height: 480.0 };
        } else {
            projection.scaling_mode = bevy::render::camera::ScalingMode::FixedHorizontal { viewport_width: 640.0 };
        }
    }
}
