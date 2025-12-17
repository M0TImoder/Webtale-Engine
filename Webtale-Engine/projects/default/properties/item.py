def get_item_data():
    """
    アイテムの定義
    heal: 回復量
    text: 使用時に表示されるテキストの2行目
    """
    return {
        "Pie": {
            "heal": 99,
            "text": ""
        },
        "I. Noodles": {
            "heal": 90,
            "text": "It's better dry."
        },
        "SnowPiece": {
            "heal": 38,
            "text": "It's a piece of snow."
        },
        "L. Hero": {
            "heal": 35,
            "text": "Increased your power!"
        }
    }

def get_initial_inventory():
    """初期インベントリの配置"""
    return [
        "Pie", 
        "I. Noodles", 
        "SnowPiece", 
        "SnowPiece",
        "L. Hero",
        "L. Hero",
        "L. Hero",
        "L. Hero"
    ]
