use bevy::prelude::*;
use rustpython_vm::PyObjectRef;

#[derive(Component)]
pub struct Cleanup;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Soul;

#[derive(Component)]
pub struct ButtonVisual {
    pub index: i32,
    pub normalTexture: Handle<Image>,
    pub selectedTexture: Handle<Image>,
}

#[derive(Component)] pub struct HpBarRed;
#[derive(Component)] pub struct HpBarYellow;
#[derive(Component)] pub struct HpText;
#[derive(Component)] pub struct LvText;
#[derive(Component)] pub struct PlayerNameText;

#[derive(Component)]
pub struct Typewriter {
    pub fullText: String,
    pub visibleChars: usize,
    pub timer: Timer,
    pub finished: bool,
}

#[derive(Component)] pub struct EnemyBody; 
#[derive(Component)] pub struct EnemyHead { pub baseY: f32, pub timer: f32 }
#[derive(Component)] pub struct ActCommands { pub commands: Vec<String> }
#[derive(Component)] pub struct MenuTextItem { pub layer: i32, pub index: i32 }
#[derive(Component)] pub struct MainDialogText;
#[derive(Component)] pub struct Vaporizing { pub scanLine: f32, pub imageHandle: Handle<Image>, pub initialY: f32 }
#[derive(Component)] pub struct DustParticle { pub velocity: Vec3, pub timer: Timer, pub maxAlpha: f32 }
#[derive(Component)] pub struct SpeechBubble;
#[derive(Component)] pub struct AttackTargetBox;
#[derive(Component)] pub struct AttackBar { pub speed: f32, pub moving: bool, pub flashTimer: Timer, pub flashState: bool }
#[derive(Component)] pub struct SliceEffect { pub timer: Timer, pub frameIndex: usize }
#[derive(Component)] pub struct PendingDamage { pub timer: Timer, pub damage: i32, pub targetPos: Vec3 }
#[derive(Component)] pub struct DamageNumber { pub timer: Timer, pub velocityY: f32, pub gravity: f32, pub startY: f32 }
#[derive(Component)] pub struct EnemyHpBar { pub lifespan: Timer, pub animation: Timer, pub startWidth: f32, pub targetWidth: f32 }
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
    pub originalPos: Vec3,
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
    Delay,      
    FadeIn,     
    Finished,   
}

#[derive(Component)]
pub struct GameOverLogo;

#[derive(Component)]
pub struct EditorWindow;

#[derive(Component)]
pub struct PythonBullet {
    pub scriptName: String,
    pub bulletData: PyObjectRef,
    pub damage: i32,
}

#[derive(Component)]
pub struct BattleScreenPreview;
