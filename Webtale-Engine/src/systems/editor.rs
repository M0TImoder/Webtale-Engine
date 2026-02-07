use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::components::{EditorWindow, BattleScreenPreview};
use crate::resources::{PlayerState, EditorState, EditorTab, EditorPreviewTexture, DanmakuPreviewTexture, BattleBox};

// エディタUI
pub fn editor_ui_system(
    mut contexts: EguiContexts,
    window_query: Query<Entity, (With<EditorWindow>, With<Window>)>,
    mut player_state: ResMut<PlayerState>,
    mut editor_state: ResMut<EditorState>,
    preview_texture: Res<EditorPreviewTexture>,
    _battle_box: ResMut<BattleBox>,
    mut bg_sprite_query: Query<&mut Visibility, With<BattleScreenPreview>>,
    danmaku_preview_texture: Res<DanmakuPreviewTexture>,
) {
    let Ok(editor_entity) = window_query.get_single() else { return };

    let _battle_texture_id = contexts.add_image(preview_texture.0.clone());
    let danmaku_texture_id = contexts.add_image(danmaku_preview_texture.0.clone());
    let ctx = contexts.ctx_for_entity_mut(editor_entity);

    for mut vis in bg_sprite_query.iter_mut() {
        if editor_state.current_tab == EditorTab::Battle {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    egui::TopBottomPanel::top("editor_tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut editor_state.current_tab, EditorTab::Battle, "Battle Screen");
            ui.selectable_value(&mut editor_state.current_tab, EditorTab::DanmakuPreview, "Danmaku Preview");
        });
    });

    egui::SidePanel::right("editor_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Danmaku Settings");
            ui.separator();

            egui::CollapsingHeader::new("Player Stats")
                .default_open(true)
                .show(ui, |ui| {
                    
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut player_state.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Level (LV):");
                        let old_lv = player_state.lv;
                        ui.add(egui::Slider::new(&mut player_state.lv, 1..=20));

                        if old_lv != player_state.lv {
                            let new_max_hp = if player_state.lv >= 20 {
                                99.0
                            } else {
                                16.0 + (player_state.lv as f32 * 4.0)
                            };
                            
                            let new_attack = 20.0 + ((player_state.lv - 1) as f32 * 2.0);

                            player_state.max_hp = new_max_hp;
                            player_state.hp = new_max_hp;
                            player_state.attack = new_attack;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Current HP:");
                        let max_hp = player_state.max_hp;
                        ui.add(egui::Slider::new(&mut player_state.hp, 0.0..=max_hp).step_by(1.0));
                    });
                    ui.label(format!("Max HP: {}", player_state.max_hp));

                    ui.separator();

                    ui.label("Movement & Combat");
                    
                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        ui.add(egui::Slider::new(&mut player_state.speed, 50.0..=400.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Attack:");
                        ui.add(egui::Slider::new(&mut player_state.attack, 10.0..=100.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Invincibility (sec):");
                        ui.add(egui::Slider::new(&mut player_state.invincibility_duration, 0.0..=5.0));
                    });
                });

            ui.separator();

            ui.heading("Bullet Pattern");
            if ui.button("Spawn Test Bullet").clicked() {
                println!("Button Clicked!"); 
            }
            
            ui.allocate_space(ui.available_size());
        });

    if editor_state.current_tab == EditorTab::DanmakuPreview {
        egui::Area::new("danmaku_preview_area".into())
            .fixed_pos(egui::Pos2::new(320.0, 45.0))
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                let size = egui::Vec2::new(640.0, 480.0);
                let response = ui.add(egui::Image::new(egui::load::SizedTexture::new(danmaku_texture_id, size)));

                 if response.clicked() || response.dragged() {
                     if let Some(pos) = response.interact_pointer_pos() {
                         
                         let image_rect = response.rect;
                         let rel_x = pos.x - image_rect.min.x;
                         let rel_y = pos.y - image_rect.min.y;
                         
                         let uv_x = rel_x / image_rect.width();
                         let uv_y = rel_y / image_rect.height();
                         
                         let world_x = (uv_x - 0.5) * 640.0;
                         let world_y = (0.5 - uv_y) * 480.0;
                         
                         println!("Preview Click: World({}, {})", world_x, world_y);
                     }
                 }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |_ui| {});
    }
}
