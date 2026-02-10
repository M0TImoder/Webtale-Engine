def player_status(cls):
    cls.__is_player_status__ = True
    return cls

@player_status
class PlayerStatus:
    name = "CHARA"
    lv = 1
    maxHp = 20.0
    hp = 20.0
    speed = 150.0
    attack = 20.0
    defense = 0.0
    invincibilityDuration = 1.0
    inventory = [
        "Pie",
        "I. Noodles",
        "SnowPiece",
        "SnowPiece",
        "L. Hero",
        "L. Hero",
        "L. Hero",
        "L. Hero"
    ]
    equippedItems = []

def getPlayerStatus():
    return {
        "name": PlayerStatus.name,
        "lv": PlayerStatus.lv,
        "maxHp": PlayerStatus.maxHp,
        "hp": PlayerStatus.hp,
        "speed": PlayerStatus.speed,
        "attack": PlayerStatus.attack,
        "defense": PlayerStatus.defense,
        "invincibilityDuration": PlayerStatus.invincibilityDuration,
        "inventory": PlayerStatus.inventory,
        "equippedItems": PlayerStatus.equippedItems
    }
