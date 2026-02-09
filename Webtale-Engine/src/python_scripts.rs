// Pythonスクリプト読み込み
use std::fs;
use std::path::{Path, PathBuf};

// プロジェクトルートパス
fn project_root(project: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("projects")
        .join(project)
}

// スクリプト読み込み
fn read_script(path: PathBuf) -> Option<String> {
    fs::read_to_string(path).ok()
}

// アイテムスクリプト
pub fn get_item_script(project: &str) -> Option<String> {
    read_script(project_root(project).join("properties").join("item.py"))
}

// プレイヤーステータススクリプト
pub fn get_player_status_script(project: &str) -> Option<String> {
    read_script(project_root(project).join("properties").join("playerStatus.py"))
}

// プロパティスクリプト
pub fn get_properties_script(project: &str) -> Option<String> {
    read_script(project_root(project).join("properties").join("properties.py"))
}

// 敵ステータススクリプト
pub fn get_enemy_status_script(project: &str) -> Option<String> {
    read_script(project_root(project).join("properties").join("enemyStatus.py"))
}

// 立ち絵スクリプト
pub fn get_tachie_script(project: &str, script_name: &str) -> Option<String> {
    if script_name.is_empty() {
        return None;
    }
    read_script(project_root(project).join("tachie").join(format!("{}.py", script_name)))
}

// フェーズAPIスクリプト
pub fn get_phase_api_script(project: &str) -> Option<String> {
    read_script(project_root(project).join("phases").join("phase_api.py"))
}

// フェーズスクリプト
pub fn get_phase_script(project: &str, phase_name: &str) -> Option<String> {
    if phase_name.is_empty() {
        return None;
    }
    read_script(project_root(project).join("phases").join(format!("{}.py", phase_name)))
}

// フェーズ一覧
pub fn list_phase_names(project: &str) -> Vec<String> {
    let phases_dir = project_root(project).join("phases");
    let entries = match fs::read_dir(phases_dir) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("py") {
            continue;
        }
        let stem = match path.file_stem().and_then(|stem| stem.to_str()) {
            Some(stem) => stem,
            None => continue,
        };
        if stem == "phase_api" {
            continue;
        }
        names.push(stem.to_string());
    }
    names
}

// 弾幕APIスクリプト
pub fn get_danmaku_api_script(project: &str) -> Option<String> {
    read_script(project_root(project).join("danmaku").join("api.py"))
}

// 弾幕スクリプト
pub fn get_danmaku_script(project: &str, script_name: &str) -> Option<String> {
    if script_name.is_empty() {
        return None;
    }
    read_script(project_root(project).join("danmaku").join(format!("{}.py", script_name)))
}
