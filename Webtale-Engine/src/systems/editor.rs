use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::components::{EditorWindow, BattleScreenPreview};
use crate::resources::{GameState, EditorState, EditorTab, EditorPreviewTexture, DanmakuPreviewTexture, BattleBox};

pub fn editorUiSystem(
    mut contexts: EguiContexts,
    windowQuery: Query<Entity, (With<EditorWindow>, With<Window>)>,
    mut gameState: ResMut<GameState>,
    mut editorState: ResMut<EditorState>,
    previewTexture: Res<EditorPreviewTexture>,
    _battleBox: ResMut<BattleBox>,
    mut bgSpriteQuery: Query<&mut Visibility, With<BattleScreenPreview>>,
    danmakuPreviewTexture: Res<DanmakuPreviewTexture>,
) {
    let Ok(editorEntity) = windowQuery.get_single() else { return };

    let _battleTextureId = contexts.add_image(previewTexture.0.clone());
    let danmakuTextureId = contexts.add_image(danmakuPreviewTexture.0.clone());
    let ctx = contexts.ctx_for_window_mut(editorEntity);

    for mut vis in bgSpriteQuery.iter_mut() {
        if editorState.currentTab == EditorTab::Battle {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    egui::TopBottomPanel::top("editor_tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut editorState.currentTab, EditorTab::Battle, "Battle Screen");
            ui.selectable_value(&mut editorState.currentTab, EditorTab::DanmakuPreview, "Danmaku Preview");
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
                        ui.text_edit_singleline(&mut gameState.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Level (LV):");
                        let oldLv = gameState.lv;
                        ui.add(egui::Slider::new(&mut gameState.lv, 1..=20));

                        if oldLv != gameState.lv {
                            let newMaxHp = if gameState.lv >= 20 {
                                99.0
                            } else {
                                16.0 + (gameState.lv as f32 * 4.0)
                            };
                            
                            let newAttack = 20.0 + ((gameState.lv - 1) as f32 * 2.0);

                            gameState.maxHp = newMaxHp;
                            gameState.hp = newMaxHp;
                            gameState.attack = newAttack;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Current HP:");
                        let maxHp = gameState.maxHp;
                        ui.add(egui::Slider::new(&mut gameState.hp, 0.0..=maxHp).step_by(1.0));
                    });
                    ui.label(format!("Max HP: {}", gameState.maxHp));

                    ui.separator();

                    ui.label("Movement & Combat");
                    
                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        ui.add(egui::Slider::new(&mut gameState.speed, 50.0..=400.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Attack:");
                        ui.add(egui::Slider::new(&mut gameState.attack, 10.0..=100.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Invincibility (sec):");
                        ui.add(egui::Slider::new(&mut gameState.invincibilityDuration, 0.0..=5.0));
                    });
                });

            ui.separator();

            ui.heading("Bullet Pattern");
            if ui.button("Spawn Test Bullet").clicked() {
                println!("Button Clicked!"); 
            }
            
            ui.allocate_space(ui.available_size());
        });

    if editorState.currentTab == EditorTab::DanmakuPreview {
        egui::Area::new("danmaku_preview_area".into())
            .fixed_pos(egui::Pos2::new(320.0, 45.0))
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                let size = egui::Vec2::new(640.0, 480.0);
                let response = ui.add(egui::Image::new(egui::load::SizedTexture::new(danmakuTextureId, size)));

                 if response.clicked() || response.dragged() {
                     if let Some(pos) = response.interact_pointer_pos() {
                         
                         let imageRect = response.rect;
                         let relX = pos.x - imageRect.min.x;
                         let relY = pos.y - imageRect.min.y;
                         
                         let uvX = relX / imageRect.width();
                         let uvY = relY / imageRect.height();
                         
                         let worldX = (uvX - 0.5) * 640.0;
                         let worldY = (0.5 - uvY) * 480.0;
                         
                         println!("Preview Click: World({}, {})", worldX, worldY);
                     }
                 }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |_ui| {});
    }
}
