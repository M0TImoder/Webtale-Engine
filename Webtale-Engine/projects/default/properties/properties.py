def properties(cls):
    cls.__is_properties__ = True
    return cls

@properties
class GameData:
    items = {
        "Pie": {
            "kind": "item",
            "heal": 99,
            "attack": 0,
            "defense": 0,
            "text": ""
        },
        "I. Noodles": {
            "kind": "item",
            "heal": 90,
            "attack": 0,
            "defense": 0,
            "text": "It's better dry."
        },
        "SnowPiece": {
            "kind": "item",
            "heal": 38,
            "attack": 0,
            "defense": 0,
            "text": "It's a piece of snow."
        },
        "L. Hero": {
            "kind": "item",
            "heal": 35,
            "attack": 0,
            "defense": 0,
            "text": "Increased your power!"
        }
    }

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
            "weapon": "",
            "armor": ""
        }
    }

    status = {
        "name": "CHARA",
        "lv": 1,
        "maxHp": 20.0,
        "hp": 20.0,
        "speed": 150.0,
        "attack": 20.0,
        "defense": 0.0,
        "invincibilityDuration": 1.0
    }
