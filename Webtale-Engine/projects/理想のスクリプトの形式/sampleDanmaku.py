@danmaku
class AttackLibrary:
    # 弾幕パターンA
    def attackA(self):
        # 3方向、速度5、角度90度で垂直攻撃
        attack.verticalAtk(count=3, speed=5, angle=90)
        yield wait(50)  # 50フレーム待機
        
        # 11個の弾、速度32、プレイヤーの位置を狙って水平攻撃
        attack.horizontalAtk(count=11, speed=32, target=ops.player)
        yield wait(10)

    # 弾幕パターンB（応用例：円形攻撃）
    def attackB(self):
        for i in range(0, 360, 30):
            attack.create_bullet(speed=3, angle=i)
        yield wait(100)
        