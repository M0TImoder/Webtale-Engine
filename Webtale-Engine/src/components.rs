use bevy::prelude::*;

#[derive(Component)]
pub struct Cleanup;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Soul;

#[derive(Component)]
pub struct ButtonVisual {
    pub index: i32,
    pub normal_texture: Handle<Image>,
    pub selected_texture: Handle<Image>,
}

#[derive(Component)] pub struct HpBarRed;
#[derive(Component)] pub struct HpBarYellow;
#[derive(Component)] pub struct HpText;
#[derive(Component)] pub struct LvText;
#[derive(Component)] pub struct PlayerNameText;

#[derive(Component)]
pub struct Typewriter {
    pub full_text: String,
    pub visible_chars: usize,
    pub timer: Timer,
    pub finished: bool,
}

#[derive(Component)] pub struct EnemyBody; 
#[derive(Component)] pub struct EnemyHead { pub base_y: f32, pub timer: f32 }
#[derive(Component)] pub struct ActCommands { pub commands: Vec<String> }
#[derive(Component)] pub struct MenuTextItem { pub layer: i32, pub index: i32 }
#[derive(Component)] pub struct MainDialogText;
#[derive(Component)] pub struct Vaporizing { pub scan_line: f32, pub image_handle: Handle<Image>, pub initial_y: f32 }
#[derive(Component)] pub struct DustParticle { pub velocity: Vec3, pub timer: Timer, pub max_alpha: f32 }
#[derive(Component)] pub struct SpeechBubble;
#[derive(Component)] pub struct AttackTargetBox;
#[derive(Component)] pub struct AttackBar { pub speed: f32, pub moving: bool, pub flash_timer: Timer, pub flash_state: bool }
#[derive(Component)] pub struct SliceEffect { pub timer: Timer, pub frame_index: usize }
#[derive(Component)] pub struct PendingDamage { pub timer: Timer, pub damage: i32, pub target_pos: Vec3 }
#[derive(Component)] pub struct DamageNumber { pub timer: Timer, pub velocity_y: f32, pub gravity: f32, pub start_y: f32 }
#[derive(Component)] pub struct EnemyHpBar { pub lifespan: Timer, pub animation: Timer, pub start_width: f32, pub target_width: f32 }
#[derive(Component)] pub struct EnemyHpBarForeground;
#[derive(Component)] pub struct BorderVisual;
#[derive(Component)] pub struct BackgroundVisual;

#[derive(Component)]
pub struct LeapFrogBullet {
    pub state: LeapFrogState,
    pub timer: Timer,
    pub velocity: Vec3,
    pub gravity: Vec3,
    pub damage: i32,
}

pub enum LeapFrogState {
    Waiting,
    Jumping,
}

#[derive(Component)]
pub struct HeartDefeated {
    pub timer: Timer,
    pub state: HeartDefeatedState,
    pub original_pos: Vec3,
}

pub enum HeartDefeatedState {
    InitialDelay,
    Cracked,
}

#[derive(Component)]
pub struct HeartShard {
    pub velocity: Vec3,
    pub gravity: f32,
}

#[derive(Component)]
pub struct GameOverSequence {
    pub timer: Timer,
    pub state: GameOverSequenceState,
}

pub enum GameOverSequenceState {
    Delay,      // 破片が飛び散った後の待機
    FadeIn,     // ロゴのフェードイン
    Finished,   // 完了
}

#[derive(Component)]
pub struct GameOverLogo;

#[derive(Component)]
pub struct EditorWindow;
