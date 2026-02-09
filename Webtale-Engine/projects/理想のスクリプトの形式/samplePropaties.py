@properties
class GameData:
    # アイテム・武器・防具の定義
    # 辞書形式を使いつつ、Pythonの型ヒントで構造を明示する
    items = {
        "Pie": {
            "kind": "item",
            "heal": 99,
            "text": ""
        },
        "I. Noodles": {
            "kind": "item",
            "heal": 90,
            "text": "It's better dry."
        },
        "SnowPiece": {
            "kind": "item",
            "heal": 38,
            "text": "It's a piece of snow."
        },
        "L. Hero": {
            "kind": "item",
            "heal": 35,
            "text": "Increased your power!"
        },
        "Knife": {
            "kind": "weapon",
            "attack": 99,
            "text": "Finaly found."
        },
        "Rocket": {
            "kind": "armor",
            "defense": 99,
            "text": "Finaly found."
        }
    }

    # 所持品と装備の状態
    inventory = {
        "items": [
            "Pie",
            "I. Noodles",
            "SnowPiece",
            "SnowPiece",
            "L. Hero",
            "L. Hero",
            "L. Hero",
            "L. Hero"
        ],
        "equipment": {
            "weapon": "Knife",
            "armor": "Rocket"
        }
    }

    # ステータス設定
    status = {
        "currentHP": 99,
        "maxHP": 99,
        "currentLevel": 20
    }

    # 音楽設定（マルチライン文字列を活用）
    music = {
        "main": r"audio/bgm_main.ogg",
        "phase1": r"audio/bgm_phase1.ogg",
        "phase2": r"audio/bgm_phase2.ogg",
        "final": r"audio/bgm_final.ogg"
    }
    