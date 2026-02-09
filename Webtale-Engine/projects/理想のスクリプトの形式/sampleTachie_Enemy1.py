@tachie
class Enemy1Tachie:
    # 静的な設定
    bodyTexture = "texture/enemy/spr_froglegs_0.png"
    headTexture = "texture/enemy/spr_froghead_0.png"
    headYOffset = 22.0
    baseX = 320.0
    baseY = 160.0
    scale = 1.0

    # 将来的な数式による揺れの定義例
    # システム側から 't' (経過時間) を渡して計算する
    def headSway(self, t):
        import math
        # A * sin(B * t) のような数式を直接記述
        speed = 2.0
        amplitude = 2.0
        return math.sin(t * speed) * amplitude
    