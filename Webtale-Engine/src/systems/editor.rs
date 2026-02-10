use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use crate::components::{EditorWindow, BattleScreenPreview};
use crate::resources::{PlayerState, EditorState, EditorTab, EditorPreviewTexture, DanmakuPreviewTexture, GameRunState};

fn build_dock_state() -> DockState<EditorTab> {
    let mut dock_state = DockState::new(vec![EditorTab::Battle, EditorTab::DanmakuPreview]);
    let surface = dock_state.main_surface_mut();
    let [top, _bottom] = surface.split_below(NodeIndex::root(), 0.8, vec![EditorTab::BottomPane]);
    let [top, _left] = surface.split_left(top, 0.2, vec![EditorTab::LeftPane]);
    let [_center, _right] = surface.split_right(top, 0.75, vec![EditorTab::Settings]);
    dock_state
}

struct EditorDockViewer<'a> {
    player_state: &'a mut PlayerState,
    editor_state: &'a mut EditorState,
    battle_texture_id: egui::TextureId,
    danmaku_texture_id: egui::TextureId,
    game_run_state: &'a mut GameRunState,
}

impl<'a> EditorDockViewer<'a> {
    fn draw_preview(
        &mut self,
        ui: &mut egui::Ui,
        texture_id: egui::TextureId,
        show_controls: bool,
        control_id: &'static str,
        enforce_aspect: bool,
    ) {
        let size = ui.available_size();
        if size.x <= 0.0 || size.y <= 0.0 {
            return;
        }

        let (container_rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
        let target_size = if enforce_aspect {
            let target_ratio = 4.0 / 3.0;
            let panel_ratio = size.x / size.y;
            if panel_ratio > target_ratio {
                let height = size.y;
                egui::vec2(height * target_ratio, height)
            } else {
                let width = size.x;
                egui::vec2(width, width / target_ratio)
            }
        } else {
            size
        };

        let preview_rect = egui::Rect::from_center_size(container_rect.center(), target_size);
        egui::Image::new(egui::load::SizedTexture::new(
            texture_id,
            egui::vec2(640.0, 480.0),
        ))
        .paint_at(ui, preview_rect);

        let pointer_pos = ui.ctx().input(|i| i.pointer.hover_pos());
        let hovered = pointer_pos.map_or(false, |pos| preview_rect.contains(pos));
        let show_overlay = show_controls && (self.editor_state.controls_pinned || hovered);

        if show_overlay {
            let button_size = egui::vec2(28.0, 22.0);
            let spacing = 6.0;
            let total_width = (button_size.x * 3.0) + (spacing * 2.0) + 36.0;
            let start_x = preview_rect.center().x - (total_width / 2.0);
            let y = preview_rect.top() + 6.0;
            let active_color = egui::Color32::from_rgb(60, 120, 220);

            egui::Area::new(control_id.into())
                .order(egui::Order::Foreground)
                .fixed_pos(egui::pos2(start_x, y))
                .movable(false)
                .interactable(true)
                .sense(egui::Sense::click())
                .fade_in(false)
                .show(ui.ctx(), |ui| {
                    ui.set_enabled(true);
                    ui.horizontal(|ui| {
                        ui.set_min_size(egui::vec2(total_width, button_size.y));
                        let play_button = if self.game_run_state.running {
                            egui::Button::new("▶").fill(active_color)
                        } else {
                            egui::Button::new("▶")
                        };
                        if ui.add_sized(button_size, play_button).clicked() {
                            self.game_run_state.running = true;
                        }

                        let stop_button = if !self.game_run_state.running {
                            egui::Button::new("⏸").fill(active_color)
                        } else {
                            egui::Button::new("⏸")
                        };
                        if ui.add_sized(button_size, stop_button).clicked() {
                            self.game_run_state.running = false;
                        }

                        if ui.add_sized(button_size, egui::Button::new("■")).clicked() {
                            self.game_run_state.running = false;
                            self.game_run_state.reset_requested = true;
                        }

                        ui.toggle_value(&mut self.editor_state.controls_pinned, "👁");
                    });
                });
        }
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Danmaku Settings");
        ui.separator();

        egui::CollapsingHeader::new("Player Stats")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.player_state.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Level (LV):");
                    let old_lv = self.player_state.lv;
                    ui.add(egui::Slider::new(&mut self.player_state.lv, 1..=20));

                    if old_lv != self.player_state.lv {
                        let new_max_hp = if self.player_state.lv >= 20 {
                            99.0
                        } else {
                            16.0 + (self.player_state.lv as f32 * 4.0)
                        };

                        let new_attack = 20.0 + ((self.player_state.lv - 1) as f32 * 2.0);

                        self.player_state.max_hp = new_max_hp;
                        self.player_state.hp = new_max_hp;
                        self.player_state.attack = new_attack;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Current HP:");
                    let max_hp = self.player_state.max_hp;
                    ui.add(egui::Slider::new(&mut self.player_state.hp, 0.0..=max_hp).step_by(1.0));
                });
                ui.label(format!("Max HP: {}", self.player_state.max_hp));

                ui.separator();

                ui.label("Movement & Combat");

                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    ui.add(egui::Slider::new(&mut self.player_state.speed, 50.0..=400.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Attack:");
                    ui.add(egui::Slider::new(&mut self.player_state.attack, 10.0..=100.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Invincibility (sec):");
                    ui.add(egui::Slider::new(&mut self.player_state.invincibility_duration, 0.0..=5.0));
                });
            });

        ui.separator();

        ui.heading("Bullet Pattern");
        if ui.button("Spawn Test Bullet").clicked() {
            println!("Button Clicked!");
        }

        ui.allocate_space(ui.available_size());
    }
}

impl TabViewer for EditorDockViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Battle => "Battle Screen".into(),
            EditorTab::DanmakuPreview => "Danmaku Preview".into(),
            EditorTab::Settings => "Danmaku Settings".into(),
            EditorTab::LeftPane => "Left Pane".into(),
            EditorTab::BottomPane => "Bottom Pane".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Battle => {
                self.editor_state.current_tab = EditorTab::Battle;
                self.draw_preview(ui, self.battle_texture_id, true, "battle_controls", true);
            }
            EditorTab::DanmakuPreview => {
                self.editor_state.current_tab = EditorTab::DanmakuPreview;
                self.editor_state.preview_active = true;
                self.draw_preview(ui, self.danmaku_texture_id, false, "danmaku_controls", false);
            }
            EditorTab::Settings => {
                self.draw_settings(ui);
            }
            EditorTab::LeftPane | EditorTab::BottomPane => {
                ui.allocate_space(ui.available_size());
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        match tab {
            EditorTab::Battle | EditorTab::DanmakuPreview | EditorTab::LeftPane | EditorTab::BottomPane => {
                [false, false]
            }
            EditorTab::Settings => [true, true],
        }
    }
}

// エディタUI
pub fn editor_ui_system(
    mut contexts: EguiContexts,
    window_query: Query<Entity, (With<EditorWindow>, With<Window>)>,
    mut player_state: ResMut<PlayerState>,
    mut editor_state: ResMut<EditorState>,
    preview_texture: Res<EditorPreviewTexture>,
    mut bg_sprite_query: Query<&mut Visibility, With<BattleScreenPreview>>,
    danmaku_preview_texture: Res<DanmakuPreviewTexture>,
    mut game_run_state: ResMut<GameRunState>,
    mut dock_state: Local<Option<DockState<EditorTab>>>,
) {
    let Ok(editor_entity) = window_query.get_single() else { return };

    let battle_texture_id = contexts.add_image(preview_texture.0.clone());
    let danmaku_texture_id = contexts.add_image(danmaku_preview_texture.0.clone());
    let ctx = contexts.ctx_for_entity_mut(editor_entity);

    if !editor_state.font_configured {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "noto_sans_jp".to_string(),
            egui::FontData::from_static(include_bytes!("../../assets/font/NotoSansJP-VariableFont_wght.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "noto_sans_jp".to_string());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "noto_sans_jp".to_string());
        ctx.set_fonts(fonts);
        editor_state.font_configured = true;
    }

    editor_state.preview_active = false;

    for mut vis in bg_sprite_query.iter_mut() {
        *vis = Visibility::Hidden;
    }

    if dock_state.is_none() {
        *dock_state = Some(build_dock_state());
    }

    let mut viewer = EditorDockViewer {
        player_state: &mut player_state,
        editor_state: &mut editor_state,
        battle_texture_id,
        danmaku_texture_id,
        game_run_state: &mut game_run_state,
    };

    DockArea::new(dock_state.as_mut().unwrap())
        .show_close_buttons(true)
        .show(ctx, &mut viewer);
}
