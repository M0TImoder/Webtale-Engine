use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;

const ORIGIN_X: f32 = -320.0;
const ORIGIN_Y: f32 = 240.0;

const COLOR_HP_RED: Color = Color::rgb(1.0, 0.0, 0.0);
const COLOR_HP_YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
const COLOR_UI_TEXT: Color = Color::WHITE;

const BUTTON_Y_GML: f32 = 432.0;
const BTN_FIGHT_X: f32 = 32.0;
const BTN_ACT_X: f32 = 185.0;
const BTN_ITEM_X: f32 = 345.0;
const BTN_MERCY_X: f32 = 500.0;

const Z_BORDER: f32 = 0.0;
const Z_BG: f32 = 1.0;
const Z_BUTTON: f32 = 2.0;
const Z_HP_BAR_BG: f32 = 2.0;
const Z_HP_BAR_FG: f32 = 3.0;
const Z_TEXT: f32 = 4.0;
const Z_ENEMY_BODY: f32 = 3.0;
const Z_ENEMY_HEAD: f32 = 4.0;
const Z_BUBBLE: f32 = 5.0;
const Z_BUBBLE_TEXT: f32 = 6.0;
const Z_ATTACK_TARGET: f32 = 5.0;
const Z_ATTACK_BAR: f32 = 6.0;
const Z_SLICE: f32 = 15.0;
const Z_DAMAGE_TEXT: f32 = 20.0; 
const Z_SOUL: f32 = 10.0;

#[derive(Component)]
struct Soul;

#[derive(Resource)]
struct GameState {
    hp: f32,
    max_hp: f32,
    lv: i32,
    name: String,
    mnfight: i32, 
    menu_coord: i32,
    bubble_timer: Timer,
    damage_display_timer: Timer,
}

#[derive(Resource)]
struct BattleBox {
    current: Rect,
    target: Rect,
}

#[derive(Component)]
struct ButtonVisual {
    index: i32,
    normal_texture: Handle<Image>,
    selected_texture: Handle<Image>,
}

#[derive(Component)]
struct HpBarRed;
#[derive(Component)]
struct HpBarYellow;
#[derive(Component)]
struct HpText;

#[derive(Component)]
struct Typewriter {
    full_text: String,
    visible_chars: usize,
    timer: Timer,
    finished: bool,
}

#[derive(Component)]
struct EnemyHead {
    base_y: f32,
    timer: f32,
}

#[derive(Component)]
struct SpeechBubble;

#[derive(Component)]
struct AttackTargetBox;

#[derive(Component)]
struct AttackBar {
    speed: f32,
    moving: bool,
}

#[derive(Component)]
struct SliceEffect {
    timer: Timer,
    frame_index: usize,
}

#[derive(Component)]
struct DamageNumber {
    timer: Timer,
    velocity_y: f32,
}

#[derive(Component)]
struct BorderVisual;
#[derive(Component)]
struct BackgroundVisual;

#[derive(Resource)]
struct GameFonts {
    main: Handle<Font>,
    dialog: Handle<Font>,
    hp_label: Handle<Font>,
    damage: Handle<Font>, 
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                title: "Undertale Engine Recreation".to_string(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GameState {
            hp: 20.0,
            max_hp: 20.0,
            lv: 1,
            name: "CHARA".to_string(),
            mnfight: 0, 
            menu_coord: 0,
            bubble_timer: Timer::from_seconds(3.0, TimerMode::Once),
            damage_display_timer: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .insert_resource(BattleBox {
            current: Rect::new(32.0, 250.0, 602.0, 385.0),
            target: Rect::new(32.0, 250.0, 602.0, 385.0),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            menu_navigation,
            soul_position_sync,
            update_box_size,
            draw_battle_box,
            draw_ui_status,
            update_button_sprites,
            animate_text,
            debug_spawn_text,
            animate_enemy_head, 
            battle_flow_control,
            attack_bar_update,
            animate_slice_effect,
            damage_number_update,
        ))
        .run();
}

fn gml_to_bevy(x: f32, y: f32) -> Vec3 {
    Vec3::new(ORIGIN_X + x, ORIGIN_Y - y, 0.0)
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let font_main = asset_server.load("Mars_Needs_Cunnilingus.ttf");
    let font_dialog = asset_server.load("8bitOperatorPlus-Bold.ttf");
    let font_hp_label = asset_server.load("8-BIT_WO.ttf");
    let font_damage = asset_server.load("hachicro.TTF");

    commands.insert_resource(GameFonts {
        main: font_main.clone(),
        dialog: font_dialog.clone(),
        hp_label: font_hp_label.clone(),
        damage: font_damage.clone(), 
    });

    let enemy_base_x = 320.0; 
    let enemy_base_y = 136.0;

    commands.spawn(SpriteBundle {
        texture: asset_server.load("spr_froglegs_0.png"),
        sprite: Sprite { color: Color::WHITE, custom_size: None, ..default() },
        transform: Transform {
            translation: gml_to_bevy(enemy_base_x, enemy_base_y) + Vec3::new(0.0, 0.0, Z_ENEMY_BODY),
            scale: Vec3::splat(2.0), 
            ..default()
        },
        ..default()
    });

    let head_y_offset = 32.0; 
    let head_pos = gml_to_bevy(enemy_base_x, enemy_base_y - head_y_offset);

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("spr_froghead_0.png"),
            sprite: Sprite { color: Color::WHITE, custom_size: None, ..default() },
            transform: Transform {
                translation: head_pos + Vec3::new(0.0, 0.0, Z_ENEMY_HEAD),
                scale: Vec3::splat(2.0), 
                ..default()
            },
            ..default()
        },
        EnemyHead { base_y: head_pos.y, timer: 0.0 },
    ));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("spr_heart_0.png"), 
            sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(16.0, 16.0)), ..default() },
            transform: Transform::from_translation(gml_to_bevy(0.0, 0.0) + Vec3::new(0.0, 0.0, Z_SOUL)),
            ..default()
        },
        Soul,
    ));

    let buttons = [
        (BTN_FIGHT_X, "spr_fightbt_0.png", "spr_fightbt_1.png", 0),
        (BTN_ACT_X,   "spr_actbt_center_0.png", "spr_actbt_center_1.png", 1),
        (BTN_ITEM_X,  "spr_itembt_0.png",  "spr_itembt_1.png",  2),
        (BTN_MERCY_X, "spr_sparebt_0.png", "spr_sparebt_1.png", 3),
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
        ));
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: Color::WHITE, ..default() },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, Z_BORDER)),
            ..default()
        },
        BorderVisual,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: Color::BLACK, ..default() },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, Z_BG)),
            ..default()
        },
        BackgroundVisual,
    ));

    let font_size = 23.0; 
    let font_style = TextStyle { font: font_main.clone(), font_size, color: COLOR_UI_TEXT };

    commands.spawn(Text2dBundle {
        text: Text::from_section("CHARA", font_style.clone()),
        text_anchor: Anchor::TopLeft,
        transform: Transform::from_translation(gml_to_bevy(30.0, 396.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
        ..default()
    });

    let lv_x = 30.0 + 85.0 + 15.0; 
    commands.spawn(Text2dBundle {
        text: Text::from_section("LV 1", font_style.clone()),
        text_anchor: Anchor::TopLeft,
        transform: Transform::from_translation(gml_to_bevy(lv_x, 396.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
        ..default()
    });

    commands.spawn(Text2dBundle {
        text: Text::from_section("HP", TextStyle { font: font_hp_label.clone(), font_size: 9.0, color: COLOR_UI_TEXT }),
        text_anchor: Anchor::TopLeft,
        transform: Transform::from_translation(gml_to_bevy(225.0, 405.0) + Vec3::new(0.0, 0.0, Z_TEXT)), 
        ..default()
    });

    let hp_bar_x = 250.0;
    let hp_bar_y = 400.0;

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: COLOR_HP_RED, anchor: Anchor::TopLeft, ..default() },
            transform: Transform::from_translation(gml_to_bevy(hp_bar_x, hp_bar_y) + Vec3::new(0.0, 0.0, Z_HP_BAR_BG)),
            ..default()
        },
        HpBarRed,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: COLOR_HP_YELLOW, anchor: Anchor::TopLeft, ..default() },
            transform: Transform::from_translation(gml_to_bevy(hp_bar_x, hp_bar_y) + Vec3::new(0.0, 0.0, Z_HP_BAR_FG)),
            ..default()
        },
        HpBarYellow,
    ));

    let hp_text_x = 250.0 + 24.0 + 15.0;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("20 / 20", font_style),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(gml_to_bevy(hp_text_x, 396.0) + Vec3::new(0.0, 0.0, Z_TEXT)),
            ..default()
        },
        HpText,
    ));
}

fn menu_navigation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.mnfight == 0 {
        if input.just_pressed(KeyCode::ArrowLeft) {
            game_state.menu_coord = (game_state.menu_coord - 1 + 4) % 4;
        }
        if input.just_pressed(KeyCode::ArrowRight) {
            game_state.menu_coord = (game_state.menu_coord + 1) % 4;
        }
        
        if input.just_pressed(KeyCode::KeyZ) {
            match game_state.menu_coord {
                0 => { 
                    game_state.mnfight = 4; 
                    let box_center = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 250.0 + (385.0-250.0)/2.0);
                    commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load("spr_target.png"),
                            sprite: Sprite { custom_size: Some(Vec2::new(566.0, 120.0)), ..default() },
                            transform: Transform::from_translation(box_center + Vec3::new(0.0, 0.0, Z_ATTACK_TARGET)),
                            ..default()
                        },
                        AttackTargetBox,
                    ));
                    let bar_start_x = gml_to_bevy(32.0, 0.0).x;
                    commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load("spr_targetchoice_1.png"),
                            sprite: Sprite { custom_size: Some(Vec2::new(14.0, 120.0)), ..default() },
                            transform: Transform::from_translation(Vec3::new(bar_start_x, box_center.y, Z_ATTACK_BAR)),
                            ..default()
                        },
                        AttackBar { speed: 420.0, moving: true },
                    ));
                },
                _ => { 
                    game_state.mnfight = 1; 
                    game_state.bubble_timer.reset();
                }
            }
        }
    }
}

fn attack_bar_update(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    game_fonts: Res<GameFonts>,
    mut query: Query<(Entity, &mut Transform, &mut AttackBar)>,
    target_box: Query<Entity, With<AttackTargetBox>>,
) {
    if game_state.mnfight != 4 { return; }

    for (bar_entity, mut transform, mut bar) in query.iter_mut() {
        if bar.moving {
            transform.translation.x += bar.speed * time.delta_seconds();

            if input.just_pressed(KeyCode::KeyZ) {
                bar.moving = false;
                
                let box_center_x = gml_to_bevy(32.0 + (602.0-32.0)/2.0, 0.0).x;
                let distance = (transform.translation.x - box_center_x).abs();
                
                let damage = if distance < 12.0 {
                    (20.0 * 2.2) as i32 
                } else {
                    let stretch = (280.0 - distance).max(0.0) / 280.0;
                    (20.0 * stretch * 2.0) as i32
                };

                let enemy_pos = gml_to_bevy(320.0, 120.0);
                
                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("spr_strike_0.png"),
                        transform: Transform {
                            translation: enemy_pos + Vec3::new(0.0, 0.0, Z_SLICE),
                            scale: Vec3::splat(2.0),
                            ..default()
                        },
                        ..default()
                    },
                    SliceEffect { timer: Timer::from_seconds(0.15, TimerMode::Repeating), frame_index: 0 },
                ));

                let damage_str = format!("{}", damage);
                let text_style_outline = TextStyle { 
                    font: game_fonts.damage.clone(), 
                    font_size: 40.0, 
                    color: Color::RED 
                };
                let text_style_main = TextStyle { 
                    font: game_fonts.damage.clone(), 
                    font_size: 40.0, 
                    color: Color::BLACK 
                };

                commands.spawn((
                    SpatialBundle {
                        transform: Transform::from_translation(enemy_pos + Vec3::new(0.0, 50.0, Z_DAMAGE_TEXT)),
                        ..default()
                    },
                    DamageNumber { timer: Timer::from_seconds(1.0, TimerMode::Once), velocity_y: 200.0 },
                )).with_children(|parent| {
                    let offsets = [
                        Vec3::new( 2.0,  0.0, 0.0), Vec3::new(-2.0,  0.0, 0.0),
                        Vec3::new( 0.0,  2.0, 0.0), Vec3::new( 0.0, -2.0, 0.0),
                        Vec3::new( 2.0,  2.0, 0.0), Vec3::new(-2.0, -2.0, 0.0),
                        Vec3::new( 2.0, -2.0, 0.0), Vec3::new(-2.0,  2.0, 0.0),
                    ];

                    for offset in offsets {
                        parent.spawn(Text2dBundle { 
                            text: Text::from_section(&damage_str, text_style_outline.clone()), 
                            transform: Transform::from_translation(offset), 
                            ..default() 
                        });
                    }
                    
                    parent.spawn(Text2dBundle { 
                        text: Text::from_section(&damage_str, text_style_main), 
                        transform: Transform::from_xyz(0.0, 0.0, 0.1), 
                        ..default() 
                    });
                });

                commands.entity(bar_entity).despawn();
                if let Ok(box_entity) = target_box.get_single() {
                    commands.entity(box_entity).despawn();
                }

                game_state.mnfight = 5; 
            }
        }
    }
}

fn animate_slice_effect(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut SliceEffect, &mut Handle<Image>)>,
) {
    for (entity, mut effect, mut texture) in query.iter_mut() {
        if effect.timer.tick(time.delta()).just_finished() {
            effect.frame_index += 1;
            if effect.frame_index > 5 {
                commands.entity(entity).despawn();
            } else {
                let path = format!("spr_strike_{}.png", effect.frame_index);
                *texture = asset_server.load(path);
            }
        }
    }
}

fn damage_number_update(
    mut commands: Commands,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber)>,
) {
    for (entity, mut transform, mut dmg) in query.iter_mut() {
        dmg.timer.tick(time.delta());
        
        transform.translation.y += dmg.velocity_y * time.delta_seconds();
        dmg.velocity_y -= 500.0 * time.delta_seconds(); 

        if dmg.timer.finished() {
            commands.entity(entity).despawn();
            game_state.mnfight = 2;
            game_state.bubble_timer.reset(); 
        }
    }
}

fn animate_enemy_head(time: Res<Time>, mut query: Query<(&mut Transform, &mut EnemyHead)>) {
    for (mut transform, mut head) in query.iter_mut() {
        head.timer += time.delta_seconds();
        let offset = (head.timer * 2.0).sin() * 2.0; 
        transform.translation.y = head.base_y + offset;
    }
}

fn battle_flow_control(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    game_fonts: Res<GameFonts>,
    time: Res<Time>,
    mut box_res: ResMut<BattleBox>,
    bubbles: Query<Entity, With<SpeechBubble>>,
    _typewriters: Query<Entity, With<Typewriter>>,
) {
    if game_state.mnfight == 1 {
        if game_state.bubble_timer.elapsed_secs() == 0.0 {
            let bubble_x = 320.0 + 60.0; 
            let bubble_y = 100.0 - 50.0;
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("spr_blcon_sm.png"),
                    sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::new(100.0, 80.0)), ..default() },
                    transform: Transform::from_translation(gml_to_bevy(bubble_x, bubble_y) + Vec3::new(0.0, 0.0, Z_BUBBLE)),
                    ..default()
                },
                SpeechBubble,
            ));
            let messages = ["Ribbit, ribbit.", "Croak.", "Hop, hop."];
            let msg = messages[rand::thread_rng().gen_range(0..messages.len())];
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("", TextStyle { font: game_fonts.dialog.clone(), font_size: 20.0, color: Color::BLACK }),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_translation(gml_to_bevy(bubble_x + 15.0, bubble_y + 10.0) + Vec3::new(0.0, 0.0, Z_BUBBLE_TEXT)),
                    ..default()
                },
                Typewriter { full_text: msg.to_string(), visible_chars: 0, timer: Timer::from_seconds(0.05, TimerMode::Repeating), finished: false },
                SpeechBubble, 
            ));
        }
        game_state.bubble_timer.tick(time.delta());
        if game_state.bubble_timer.finished() {
            for entity in bubbles.iter() { commands.entity(entity).despawn_recursive(); }
            game_state.mnfight = 2;
            box_res.target = Rect::new(217.0, 125.0, 417.0, 385.0);
        }
    }
}

fn soul_position_sync(
    game_state: Res<GameState>,
    mut soul_query: Query<&mut Transform, With<Soul>>,
) {
    let mut transform = soul_query.single_mut();
    if game_state.mnfight == 0 {
        let offset_x = 8.0 + 8.0; 
        let offset_y = 14.0 + 8.0; 
        let target_x = match game_state.menu_coord {
            0 => BTN_FIGHT_X, 1 => BTN_ACT_X, 2 => BTN_ITEM_X, 3 => BTN_MERCY_X, _ => 0.0,
        };
        let pos = gml_to_bevy(target_x + offset_x, BUTTON_Y_GML + offset_y);
        transform.translation = pos + Vec3::new(0.0, 0.0, Z_SOUL);
    }
}

fn update_box_size(
    mut box_res: ResMut<BattleBox>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        if box_res.target.width() > 300.0 { box_res.target = Rect::new(217.0, 125.0, 417.0, 385.0); } 
        else { box_res.target = Rect::new(32.0, 250.0, 602.0, 385.0); }
    }
    let speed = 15.0 * time.delta_seconds();
    box_res.current.min.x += (box_res.target.min.x - box_res.current.min.x) * speed;
    box_res.current.min.y += (box_res.target.min.y - box_res.current.min.y) * speed;
    box_res.current.max.x += (box_res.target.max.x - box_res.current.max.x) * speed;
    box_res.current.max.y += (box_res.target.max.y - box_res.current.max.y) * speed;
}

fn draw_battle_box(
    box_res: Res<BattleBox>,
    mut border: Query<&mut Transform, (With<BorderVisual>, Without<BackgroundVisual>)>,
    mut border_spr: Query<&mut Sprite, (With<BorderVisual>, Without<BackgroundVisual>)>,
    mut bg: Query<&mut Transform, (With<BackgroundVisual>, Without<BorderVisual>)>,
    mut bg_spr: Query<&mut Sprite, (With<BackgroundVisual>, Without<BorderVisual>)>,
) {
    let b = &box_res.current;
    let bevy_left = ORIGIN_X + b.min.x;
    let bevy_right = ORIGIN_X + b.max.x;
    let bevy_top = ORIGIN_Y - b.min.y; 
    let bevy_bottom = ORIGIN_Y - b.max.y;
    let width = bevy_right - bevy_left;
    let height = bevy_top - bevy_bottom;
    let center_x = bevy_left + width / 2.0;
    let center_y = bevy_bottom + height / 2.0;

    if let Ok(mut t) = border.get_single_mut() { t.translation.x = center_x; t.translation.y = center_y; }
    if let Ok(mut s) = border_spr.get_single_mut() { s.custom_size = Some(Vec2::new(width + 10.0, height + 10.0)); }
    if let Ok(mut t) = bg.get_single_mut() { t.translation.x = center_x; t.translation.y = center_y; }
    if let Ok(mut s) = bg_spr.get_single_mut() { s.custom_size = Some(Vec2::new(width, height)); }
}

fn draw_ui_status(
    game_state: Res<GameState>,
    mut red_bar: Query<&mut Sprite, (With<HpBarRed>, Without<HpBarYellow>)>,
    mut yel_bar: Query<&mut Sprite, (With<HpBarYellow>, Without<HpBarRed>)>,
    mut text: Query<(&mut Text, &mut Transform), With<HpText>>,
) {
    let bar_scale = 1.2; let height = 20.0;   
    if let Ok(mut s) = red_bar.get_single_mut() { s.custom_size = Some(Vec2::new(game_state.max_hp * bar_scale, height)); }
    if let Ok(mut s) = yel_bar.get_single_mut() { s.custom_size = Some(Vec2::new(game_state.hp * bar_scale, height)); }
    if let Ok((mut t, mut trans)) = text.get_single_mut() {
        t.sections[0].value = format!("{:.0} / {:.0}", game_state.hp, game_state.max_hp);
        let visual_hp_bar_x = 250.0;
        let text_x = visual_hp_bar_x + (game_state.max_hp * bar_scale) + 15.0;
        trans.translation = gml_to_bevy(text_x, 401.0) + Vec3::new(0.0, 0.0, Z_TEXT);
    }
}

fn update_button_sprites(
    game_state: Res<GameState>,
    mut query: Query<(&ButtonVisual, &mut Handle<Image>)>,
) {
    for (btn, mut texture_handle) in query.iter_mut() {
        if game_state.mnfight == 0 && btn.index == game_state.menu_coord {
            *texture_handle = btn.selected_texture.clone();
        } else {
            *texture_handle = btn.normal_texture.clone();
        }
    }
}

fn debug_spawn_text(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    game_fonts: Res<GameFonts>, 
    box_res: Res<BattleBox>,
    old_text: Query<Entity, With<Typewriter>>, 
) {
    if input.just_pressed(KeyCode::KeyT) {
        for entity in old_text.iter() { commands.entity(entity).despawn(); }
        let text_content = "* Froggit hopped close!";
        let b = &box_res.target;
        let start_x = b.min.x + 12.0; let start_y = b.min.y + 14.0; 
        commands.spawn((
            Text2dBundle {
                text: Text::from_section("", TextStyle { font: game_fonts.dialog.clone(), font_size: 32.0, color: Color::WHITE }),
                text_anchor: Anchor::TopLeft,
                transform: Transform::from_translation(gml_to_bevy(start_x, start_y) + Vec3::new(0.0, 0.0, Z_TEXT)),
                ..default()
            },
            Typewriter { full_text: text_content.to_string(), visible_chars: 0, timer: Timer::from_seconds(0.05, TimerMode::Repeating), finished: false },
        ));
    }
}

fn animate_text(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Typewriter, &mut Text)>,
) {
    for (mut writer, mut text) in query.iter_mut() {
        if writer.finished { continue; }
        if input.just_pressed(KeyCode::KeyX) {
            writer.visible_chars = writer.full_text.chars().count();
            text.sections[0].value = writer.full_text.clone();
            writer.finished = true; continue;
        }
        if writer.timer.tick(time.delta()).just_finished() {
            let char_count = writer.full_text.chars().count();
            if writer.visible_chars < char_count {
                writer.visible_chars += 1;
                let displayed: String = writer.full_text.chars().take(writer.visible_chars).collect();
                text.sections[0].value = displayed;
            } else { writer.finished = true; }
        }
    }
}
