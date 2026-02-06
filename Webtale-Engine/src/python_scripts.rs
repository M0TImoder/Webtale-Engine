const DEFAULT_ITEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/projects/default/properties/item.py"));
const DEFAULT_PLAYER_STATUS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/projects/default/properties/playerStatus.py"));
const DEFAULT_ENEMY_STATUS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/projects/default/properties/enemyStatus.py"));
const DEFAULT_PHASE_API: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/projects/default/phases/phase_api.py"));
const DEFAULT_PHASE1: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/projects/default/phases/phase1.py"));
const DEFAULT_PHASE_EXAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/projects/default/phases/PhaseExample.py"));
const DEFAULT_DANMAKU_API: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/projects/default/danmaku/api.py"));
const DEFAULT_DANMAKU_FROG_JUMP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/projects/default/danmaku/frogJump.py"));

pub fn get_item_script(project: &str) -> Option<&'static str> {
    match project {
        "default" => Some(DEFAULT_ITEM),
        _ => None,
    }
}

pub fn get_player_status_script(project: &str) -> Option<&'static str> {
    match project {
        "default" => Some(DEFAULT_PLAYER_STATUS),
        _ => None,
    }
}

pub fn get_enemy_status_script(project: &str) -> Option<&'static str> {
    match project {
        "default" => Some(DEFAULT_ENEMY_STATUS),
        _ => None,
    }
}

pub fn get_phase_api_script(project: &str) -> Option<&'static str> {
    match project {
        "default" => Some(DEFAULT_PHASE_API),
        _ => None,
    }
}

pub fn get_phase_script(project: &str, phase_name: &str) -> Option<&'static str> {
    match (project, phase_name) {
        ("default", "phase1") => Some(DEFAULT_PHASE1),
        ("default", "PhaseExample") => Some(DEFAULT_PHASE_EXAMPLE),
        _ => None,
    }
}

pub fn list_phase_names(project: &str) -> Vec<String> {
    match project {
        "default" => vec!["phase1".to_string(), "PhaseExample".to_string()],
        _ => vec![],
    }
}

pub fn get_danmaku_api_script(project: &str) -> Option<&'static str> {
    match project {
        "default" => Some(DEFAULT_DANMAKU_API),
        _ => None,
    }
}

pub fn get_danmaku_script(project: &str, script_name: &str) -> Option<&'static str> {
    match (project, script_name) {
        ("default", "frogJump") => Some(DEFAULT_DANMAKU_FROG_JUMP),
        _ => None,
    }
}
