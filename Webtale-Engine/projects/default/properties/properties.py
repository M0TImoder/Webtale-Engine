def get_initial_properties():
    return {
        # 基本ステータス
        "name": "CHARA",
        "lv": 1,
        "max_hp": 20.0,
        "hp": 20.0, # 現在HP
        
        # 戦闘パラメータ
        "speed": 150.0,     # 移動速度
        "attack": 20.0,     # 攻撃力 (装備品込みの値を想定)
        "invincibility_duration": 1.0, # 無敵時間(秒)

        # 敵のステータス
        "enemy_hp": 30,
        "enemy_max_hp": 30,
        "enemy_def": 0,

        # 初期所持品
        "inventory": [
            "Pie", 
            "I. Noodles",
            "SnowPiece", 
            "SnowPiece",
            "Pie", 
            "Pie", 
            "Pie", 
            "Pie"
        ]
    }
