use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use std::fs;
use std::path::{Path, PathBuf};
use crate::components::{EditorWindow, BattleScreenPreview};
use crate::resources::{PlayerState, EditorState, EditorTab, EditorPreviewTexture, DanmakuPreviewTexture, GameRunState, ProjectBrowserState};

fn build_dock_state() -> DockState<EditorTab> {
    let mut dock_state = DockState::new(vec![EditorTab::Battle, EditorTab::DanmakuPreview]);
    let surface = dock_state.main_surface_mut();
    let [top, _bottom] = surface.split_below(NodeIndex::root(), 0.8, vec![EditorTab::BottomPane]);
    let [top, _left] = surface.split_left(top, 0.2, vec![EditorTab::LeftPane]);
    let [_center, _right] = surface.split_right(top, 0.75, vec![EditorTab::Settings]);
    dock_state
}

fn projects_root() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("projects")
}

fn set_current_project(project_state: &mut ProjectBrowserState, path: PathBuf) {
    project_state.current_project = Some(path.clone());
    project_state.selected_folder = Some(path.clone());
    project_state.selected_file = None;
    project_state.renaming_file = None;
    project_state.expanded_folders.insert(path.clone());
    project_state.recent_projects.retain(|recent| recent != &path);
    project_state.recent_projects.insert(0, path);
    project_state.recent_projects.truncate(10);
}

fn path_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn draw_directory_tree(ui: &mut egui::Ui, path: &Path, project_state: &mut ProjectBrowserState) {
    let selected = project_state.selected_folder.as_ref().map(|p| p == path).unwrap_or(false);
    let is_open = project_state.expanded_folders.contains(path);
    let mut toggle_clicked = false;

    ui.horizontal(|ui| {
        let toggle_icon = if is_open { "📂" } else { "📁" };
        if ui.small_button(toggle_icon).clicked() {
            toggle_clicked = true;
        }
        let label_text = path_label(path);
        let label = if selected {
            egui::RichText::new(label_text).strong()
        } else {
            egui::RichText::new(label_text)
        };
        let label = egui::Label::new(label).truncate();
        if ui.add(label).clicked() {
            project_state.selected_folder = Some(path.to_path_buf());
            project_state.selected_file = None;
        }
    });

    let is_open = if toggle_clicked { !is_open } else { is_open };
    if is_open {
        project_state.expanded_folders.insert(path.to_path_buf());
    } else {
        project_state.expanded_folders.remove(path);
    }

    if !is_open {
        return;
    }

    let mut child_dirs = Vec::new();
    let mut child_files = Vec::new();
    let mut read_error = None;

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        read_error = Some(err.to_string());
                        break;
                    }
                };
                let file_type = match entry.file_type() {
                    Ok(file_type) => file_type,
                    Err(err) => {
                        read_error = Some(err.to_string());
                        break;
                    }
                };
                let entry_path = entry.path();
                if file_type.is_dir() {
                    child_dirs.push(entry_path);
                } else {
                    child_files.push(entry_path);
                }
            }
        }
        Err(err) => read_error = Some(err.to_string()),
    }

    ui.indent(path.to_string_lossy().to_string(), |ui| {
        if let Some(err) = read_error {
            ui.label(format!("読み込み失敗: {}", err));
            return;
        }

        child_dirs.sort_by(|a, b| path_label(a).cmp(&path_label(b)));
        child_files.sort_by(|a, b| path_label(a).cmp(&path_label(b)));
        for child in child_dirs {
            draw_directory_tree(ui, &child, project_state);
        }
        for child in child_files {
            let file_label = path_label(&child);
            ui.horizontal(|ui| {
                ui.label("📄");
                let label = egui::Label::new(file_label).truncate();
                if ui.add(label).clicked() {
                    project_state.selected_folder = Some(path.to_path_buf());
                    project_state.selected_file = Some(child.clone());
                    project_state.renaming_file = None;
                }
            });
        }
    });
}

fn draw_file_explorer(ui: &mut egui::Ui, project_state: &mut ProjectBrowserState) {
    let mut tree_width = project_state.tree_width;
    let min_tree_width = 140.0;
    let min_file_width = 200.0;
    let handle_width = 6.0;
    let available = ui.available_size();
    let max_tree_width = (available.x - handle_width - min_file_width).max(min_tree_width);
    tree_width = tree_width.clamp(min_tree_width, max_tree_width);

    ui.allocate_ui_with_layout(
        available,
        egui::Layout::left_to_right(egui::Align::Min),
        |ui| {
        ui.set_min_height(available.y);
        let height = ui.available_height();
        ui.allocate_ui_with_layout(
            egui::vec2(tree_width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("project_tree_scroll")
                    .show(ui, |ui| {
                        if let Some(project_root) = project_state.current_project.clone() {
                            draw_directory_tree(ui, &project_root, project_state);
                        } else {
                            ui.label("プロジェクトがロードされていません");
                        }
                    });
            },
        );

        let (handle_rect, handle_response) =
            ui.allocate_exact_size(egui::vec2(handle_width, height), egui::Sense::drag());
        let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let center_x = handle_rect.center().x;
        ui.painter().line_segment(
            [
                egui::pos2(center_x, handle_rect.top()),
                egui::pos2(center_x, handle_rect.bottom()),
            ],
            egui::Stroke::new(1.0, stroke_color),
        );
        if handle_response.hovered() || handle_response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        if handle_response.dragged() {
            tree_width += handle_response.drag_delta().x;
        }

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
            ui.vertical(|ui| {
                let selected_text = project_state
                    .selected_file
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or_default();
                let label = egui::Label::new(selected_text).truncate();
                ui.add_sized(egui::vec2(ui.available_width(), 20.0), label);
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("project_files_scroll")
                    .show(ui, |ui| {
                        let selected_folder = project_state
                            .selected_folder
                            .clone()
                            .or(project_state.current_project.clone());

                        let Some(folder) = selected_folder else {
                            ui.label("ここにプロジェクトファイルが表示されます");
                            return;
                        };

                        if !folder.is_dir() {
                            ui.label("フォルダが見つかりません");
                            return;
                        }

                        let mut folders = Vec::new();
                        let mut files = Vec::new();
                        let mut read_error = None;

                        match fs::read_dir(&folder) {
                            Ok(entries) => {
                                for entry in entries {
                                    let entry = match entry {
                                        Ok(entry) => entry,
                                        Err(err) => {
                                            read_error = Some(err.to_string());
                                            break;
                                        }
                                    };
                                    let file_type = match entry.file_type() {
                                        Ok(file_type) => file_type,
                                        Err(err) => {
                                            read_error = Some(err.to_string());
                                            break;
                                        }
                                    };
                                    let name = entry.file_name().to_string_lossy().to_string();
                                    if file_type.is_dir() {
                                        folders.push(name);
                                    } else {
                                        files.push(name);
                                    }
                                }
                            }
                            Err(err) => read_error = Some(err.to_string()),
                        }

                        if let Some(err) = read_error {
                            ui.label(format!("読み込み失敗: {}", err));
                            return;
                        }

                        folders.sort();
                        files.sort();

                        if folders.is_empty() && files.is_empty() {
                            ui.label("空のフォルダです");
                            return;
                        }

                        let mut entries = Vec::new();
                        for name in folders {
                            entries.push((true, folder.join(&name), name));
                        }
                        for name in files {
                            entries.push((false, folder.join(&name), name));
                        }

                        let tile_width = 120.0;
                        let tile_height = 92.0;
                        let columns = ((ui.available_width() / tile_width).floor() as usize).max(1);
                        let mut col_index = 0;

                        let mut commit_rename = None;
                        let mut cancel_rename = false;
                        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));

                        egui::Grid::new("project_files_grid")
                            .spacing(egui::vec2(12.0, 12.0))
                            .show(ui, |ui| {
                                for (is_dir, path, name) in entries {
                                    let icon = if is_dir { "📁" } else { "📄" };
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(tile_width, tile_height),
                                        egui::Sense::click(),
                                    );
                                    let is_renaming = project_state
                                        .renaming_file
                                        .as_ref()
                                        .map(|target| target == &path)
                                        .unwrap_or(false);

                                    ui.allocate_ui_at_rect(rect, |ui| {
                                        ui.with_layout(
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                ui.scope(|ui| {
                                                    let mut style = ui.style().as_ref().clone();
                                                    style.interaction.selectable_labels = false;
                                                    ui.set_style(style);
                                                    ui.label(egui::RichText::new(icon).size(32.0));
                                                });

                                                if is_renaming {
                                                    ui.horizontal_centered(|ui| {
                                                        let editor = egui::TextEdit::singleline(
                                                            &mut project_state.rename_buffer,
                                                        )
                                                        .desired_width(tile_width - 32.0);
                                                        let editor_response = ui.add(editor);
                                                        if project_state.rename_focus_requested {
                                                            editor_response.request_focus();
                                                            project_state.rename_focus_requested = false;
                                                        }
                                                        ui.label(&project_state.rename_extension);
                                                        if editor_response.has_focus() && enter_pressed {
                                                            commit_rename = Some(path.clone());
                                                        } else if editor_response.lost_focus()
                                                            && !ui.input(|i| i.pointer.any_down())
                                                        {
                                                            commit_rename = Some(path.clone());
                                                        }
                                                    });
                                                } else {
                                                    ui.scope(|ui| {
                                                        let mut style = ui.style().as_ref().clone();
                                                        style.interaction.selectable_labels = false;
                                                        ui.set_style(style);
                                                        let name_label = egui::Label::new(
                                                            egui::RichText::new(&name).size(12.0),
                                                        )
                                                        .truncate();
                                                        ui.add(name_label);
                                                    });
                                                }
                                            },
                                        );
                                    });

                                    if response.double_clicked() && !is_dir {
                                        project_state.selected_file = Some(path.clone());
                                        project_state.renaming_file = Some(path.clone());
                                        let stem = Path::new(&name)
                                            .file_stem()
                                            .map(|v| v.to_string_lossy().to_string())
                                            .unwrap_or_else(|| name.clone());
                                        let ext = Path::new(&name)
                                            .extension()
                                            .map(|v| format!(".{}", v.to_string_lossy()))
                                            .unwrap_or_default();
                                        project_state.rename_buffer = stem;
                                        project_state.rename_extension = ext;
                                        project_state.rename_focus_requested = true;
                                    } else if response.clicked() {
                                        if is_dir {
                                            project_state.selected_folder = Some(path.clone());
                                            project_state.selected_file = None;
                                            project_state.renaming_file = None;
                                            project_state.expanded_folders.insert(path);
                                        } else {
                                            project_state.selected_file = Some(path.clone());
                                            project_state.renaming_file = None;
                                        }
                                    }

                                    if escape_pressed && is_renaming {
                                        cancel_rename = true;
                                    }

                                    col_index += 1;
                                    if col_index >= columns {
                                        col_index = 0;
                                        ui.end_row();
                                    }
                                }
                                if col_index != 0 {
                                    ui.end_row();
                                }
                            });

                        if cancel_rename {
                            project_state.renaming_file = None;
                        }

                        if let Some(old_path) = commit_rename {
                            let base = project_state.rename_buffer.trim();
                            if base.is_empty() {
                                println!("Warning: empty file name");
                            } else {
                                let new_name = format!("{}{}", base, project_state.rename_extension);
                                let new_path = old_path.with_file_name(new_name);
                                if new_path != old_path {
                                    if let Err(err) = fs::rename(&old_path, &new_path) {
                                        println!("Warning: rename failed: {}", err);
                                    } else {
                                        project_state.selected_file = Some(new_path);
                                    }
                                }
                            }
                            project_state.renaming_file = None;
                        }
                    });
            });
            },
        );
    },
    );

    project_state.tree_width = tree_width.clamp(min_tree_width, max_tree_width);
}

fn draw_menu_bar(ctx: &egui::Context, project_state: &mut ProjectBrowserState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("ファイル(F)", |ui| {
                ui.menu_button("プロジェクトをロード", |ui| {
                    if ui.button("新規プロジェクトをロード").clicked() {
                        let mut dialog = rfd::FileDialog::new();
                        let projects_dir = projects_root();
                        if projects_dir.is_dir() {
                            dialog = dialog.set_directory(projects_dir);
                        }
                        if let Some(folder) = dialog.pick_folder() {
                            set_current_project(project_state, folder);
                        }
                        ui.close_menu();
                    }

                    let recent = project_state.recent_projects.clone();
                    if !recent.is_empty() {
                        ui.separator();
                        for path in recent {
                            let label = path.to_string_lossy().to_string();
                            if ui.button(label).clicked() {
                                set_current_project(project_state, path);
                                ui.close_menu();
                            }
                        }
                    }
                });
            });
        });
    });
}

struct EditorDockViewer<'a> {
    player_state: &'a mut PlayerState,
    editor_state: &'a mut EditorState,
    battle_texture_id: egui::TextureId,
    danmaku_texture_id: egui::TextureId,
    game_run_state: &'a mut GameRunState,
    project_state: &'a mut ProjectBrowserState,
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
            EditorTab::LeftPane => {
                ui.allocate_space(ui.available_size());
            }
            EditorTab::BottomPane => {
                draw_file_explorer(ui, self.project_state);
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
    mut project_state: ResMut<ProjectBrowserState>,
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

    draw_menu_bar(ctx, &mut project_state);
    let mut viewer = EditorDockViewer {
        player_state: &mut player_state,
        editor_state: &mut editor_state,
        battle_texture_id,
        danmaku_texture_id,
        game_run_state: &mut game_run_state,
        project_state: &mut project_state,
    };
    egui::CentralPanel::default().show(ctx, |ui| {
        DockArea::new(dock_state.as_mut().unwrap())
            .show_close_buttons(true)
            .show_inside(ui, &mut viewer);
    });
}
