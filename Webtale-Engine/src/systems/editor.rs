use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::path::Path;
use crate::components::{EditorWindow, BattleScreenPreview, EditorPreviewElement, EditorPreviewUI, EditorPreviewText};
use crate::resources::{GameState, EditorState, EditorTab, EditorPreviewTexture, DanmakuPreviewTexture, BattleBox, GameFonts};

pub fn configureEguiFonts(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    let mut fonts = egui::FontDefinitions::default();
    
    let font_candidates = [
        "assets/font/JF-Dot-Shinonome14.ttf",
        "../assets/font/JF-Dot-Shinonome14.ttf",
        "font/JF-Dot-Shinonome14.ttf",
        "C:/Windows/Fonts/msgothic.ttc",
        "C:/Windows/Fonts/meiryo.ttc",
    ];

    let mut font_data_loaded = None;
    let mut loaded_path = "";

    println!("[Editor] Searching for fonts...");

    for path_str in font_candidates.iter() {
        let path = Path::new(path_str);
        if path.exists() {
            if let Ok(data) = std::fs::read(path) {
                font_data_loaded = Some(data);
                loaded_path = path_str;
                println!("[Editor] Font loaded: {}", path_str);
                break;
            }
        }
    }

    if let Some(font_data) = font_data_loaded {
        fonts.font_data.insert(
            "Japanese".to_owned(),
            egui::FontData::from_owned(font_data),
        );
        
        if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            vec.insert(0, "Japanese".to_owned());
        }
        if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            vec.insert(0, "Japanese".to_owned());
        }
        
        ctx.set_fonts(fonts);
    } else {
        eprintln!("!! CRITICAL WARNING: No suitable font found. Japanese text may be garbled.");
    }
}

pub fn editorUiSystem(
    mut contexts: EguiContexts,
    windowQuery: Query<Entity, (With<EditorWindow>, With<Window>)>,
    mut gameState: ResMut<GameState>,
    mut editorState: ResMut<EditorState>,
    _previewTexture: Res<EditorPreviewTexture>,
    _battleBox: ResMut<BattleBox>,
    gameFonts: Res<GameFonts>, 
    
    mut battleScreenQuery: Query<&mut Visibility, (
        With<BattleScreenPreview>, 
        Without<EditorPreviewElement>
    )>,

    mut danmakuScreenQuery: Query<&mut Visibility, (
        With<BattleScreenPreview>,
        With<EditorPreviewElement> 
    )>,
    
    _danmakuPreviewTexture: Res<DanmakuPreviewTexture>,
    
    mut editorUiQuery: Query<&mut Visibility, (
        With<EditorPreviewUI>, 
        Without<BattleScreenPreview>,
        Without<EditorPreviewText> 
    )>,

    mut editorTextQuery: Query<(&mut Visibility, &mut Text), (
        With<EditorPreviewText>, 
        Without<BattleScreenPreview>,
        Without<EditorPreviewUI>
    )>,
) {
    let Ok(editorEntity) = windowQuery.get_single() else { return };
    let ctx = contexts.ctx_for_window_mut(editorEntity);

    for mut vis in battleScreenQuery.iter_mut() {
        *vis = if editorState.currentTab == EditorTab::Battle { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut vis in danmakuScreenQuery.iter_mut() {
        *vis = if editorState.currentTab == EditorTab::DanmakuPreview { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut vis in editorUiQuery.iter_mut() {
        *vis = if editorState.showUI { Visibility::Inherited } else { Visibility::Hidden };
    }

    for (mut vis, mut text) in editorTextQuery.iter_mut() {
        *vis = if editorState.showText { Visibility::Inherited } else { Visibility::Hidden };
        
        let (targetFont, targetSize) = if editorState.useJapaneseFont {
            (gameFonts.japanese.clone(), 26.0) 
        } else {
            (gameFonts.dialog.clone(), 32.0)
        };
        
        if text.sections[0].style.font != targetFont { text.sections[0].style.font = targetFont; }
        if text.sections[0].style.font_size != targetSize { text.sections[0].style.font_size = targetSize; }

        let rawLines: Vec<&str> = editorState.previewText.lines().collect();
        let mut formattedString = String::new();
        
        for (i, line) in rawLines.iter().enumerate() {
            if i > 0 { formattedString.push('\n'); }
            if !line.trim().is_empty() && !line.starts_with("*") {
                formattedString.push_str("  "); 
                formattedString.push_str(line);
            } else {
                formattedString.push_str(line);
            }
        }
        if text.sections[0].value != formattedString { text.sections[0].value = formattedString; }
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
            if editorState.currentTab == EditorTab::DanmakuPreview {
                ui.heading("Danmaku Editor Settings");
                ui.separator();
                
                ui.label("Display Settings:");
                ui.checkbox(&mut editorState.showText, "Show Text Box Content");
                ui.checkbox(&mut editorState.showUI, "Show Commands");
                
                ui.separator();

                if editorState.showText {
                    ui.label("Box Text:");
                    
                    ui.horizontal(|ui| {
                        ui.label("Font:");
                        ui.selectable_value(&mut editorState.useJapaneseFont, false, "English (8bit)");
                        ui.selectable_value(&mut editorState.useJapaneseFont, true, "Japanese (Shinonome)");
                    });

                    ui.add_space(5.0);
                    if ui.button("Open Text Editor (Dialog)").clicked() {
                        editorState.isEditingText = true;
                        editorState.tempEditingText = editorState.previewText.clone();
                    }
                    
                    ui.label("Current Text Preview:");
                    egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                         ui.label(&editorState.previewText);
                    });
                    
                    ui.separator();
                }
            } else {
                ui.heading("Battle Settings");
                ui.separator();
            }

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
                            let newMaxHp = if gameState.lv >= 20 { 99.0 } else { 16.0 + (gameState.lv as f32 * 4.0) };
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

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |_ui| {});

    if editorState.isEditingText {
        egui::Window::new("Edit Box Text")
            .collapsible(false)
            .resizable(true)
            .default_size(egui::vec2(500.0, 300.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.heading("Enter Text:");
                ui.label("Supports multi-line input.");
                
                ui.add(egui::TextEdit::multiline(&mut editorState.tempEditingText)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(10)
                    .lock_focus(true)
                );

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Apply & Close").clicked() {
                        editorState.previewText = editorState.tempEditingText.clone();
                        editorState.isEditingText = false;
                    }
                    if ui.button("Cancel").clicked() {
                        editorState.isEditingText = false;
                    }
                });
            });
    }
}
