def item_data(cls):
    cls.__is_item_data__ = True
    return cls

@item_data
class ItemData:
    items = {
        "Pie": {
            "heal": 99,
            "attack": 0,
            "defense": 0,
            "text": ""
        },
        "I. Noodles": {
            "heal": 90,
            "attack": 0,
            "defense": 0,
            "text": "It's better dry."
        },
        "SnowPiece": {
            "heal": 38,
            "attack": 0,
            "defense": 0,
            "text": "It's a piece of snow."
        },
        "L. Hero": {
            "heal": 35,
            "attack": 0,
            "defense": 0,
            "text": "Increased your power!"
        }
    }

def getItemData():
    return ItemData.items
