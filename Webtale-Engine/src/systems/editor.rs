use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::components::EditorWindow;
use crate::resources::GameState;

pub fn editor_ui_system(
    mut contexts: EguiContexts,
    window_query: Query<Entity, (With<EditorWindow>, With<Window>)>,
    mut game_state: ResMut<GameState>,
) {
    let Ok(editor_entity) = window_query.get_single() else { return };

    let ctx = contexts.ctx_for_window_mut(editor_entity);

    egui::SidePanel::right("editor_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Danmaku Settings");
            ui.separator();

            egui::CollapsingHeader::new("Player Stats")
                .default_open(true)
                .show(ui, |ui| {
                    
                    ui.horizontal(|ui| {
                        ui.label("Level (LV):");
                        let old_lv = game_state.lv;
                        ui.add(egui::Slider::new(&mut game_state.lv, 1..=20));

                        if old_lv != game_state.lv {
                            let new_max_hp = if game_state.lv >= 20 {
                                99.0
                            } else {
                                16.0 + (game_state.lv as f32 * 4.0)
                            };
                            
                            let new_attack = 20.0 + ((game_state.lv - 1) as f32 * 2.0);

                            game_state.max_hp = new_max_hp;
                            game_state.hp = new_max_hp;
                            game_state.attack = new_attack;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Current HP:");
                        let max_hp = game_state.max_hp;
                        ui.add(egui::Slider::new(&mut game_state.hp, 0.0..=max_hp).step_by(1.0));
                    });
                    ui.label(format!("Max HP: {}", game_state.max_hp));

                    ui.separator();

                    ui.label("Movement & Combat");
                    
                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        ui.add(egui::Slider::new(&mut game_state.speed, 50.0..=400.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Attack:");
                        ui.add(egui::Slider::new(&mut game_state.attack, 10.0..=100.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Invincibility (sec):");
                        ui.add(egui::Slider::new(&mut game_state.invincibility_duration, 0.0..=5.0));
                    });
                });

            ui.separator();

            ui.heading("Bullet Pattern");
            if ui.button("Spawn Test Bullet").clicked() {
                println!("Button Clicked!"); 
            }
            
            ui.allocate_space(ui.available_size());
        });
}
