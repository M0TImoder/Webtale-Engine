from api import Bullet

# 初期設定データ
def init():
    return {
        "texture_wait": "enemy/spr_frogbullet_stop.png",
        "box": [217.0, 125.0, 417.0, 385.0]
    }

def spawn():
    return FrogBullet()

class FrogBullet(Bullet):
    def __init__(self):
        super().__init__()
        self.state = "waiting"
        self.timer = 0.5 + self.random(0.5)
        self.damage = 4
        
        # 重力設定
        grav_speed = 0.4 * 30.0 * 30.0
        self.gx, self.gy = 0, 0
        
        rad = 280.0 * (3.14159 / 180.0)
        self.gx = grav_speed * 1.0 * (1.0 if abs(rad) < 0.001 else 1.0) # cos/sin計算はapi側でやるほうが楽だが一旦手動計算の値を引き継ぐ
        # apiのメソッド使って計算し直す
        import math
        self.gx = grav_speed * math.cos(math.radians(280.0))
        self.gy = grav_speed * math.sin(math.radians(280.0))

    def update(self, dt):
        if self.state == "waiting":
            self.timer -= dt
            if self.timer <= 0:
                self.jump()
                
        elif self.state == "jumping":
            # 重力加算
            self.vx += self.gx * dt
            self.vy += self.gy * dt

    def jump(self):
        self.state = "jumping"
        self.set_texture("enemy/spr_frogbullet_go.png")
        
        # ジャンプ方向と速度
        angle = 145.0 - self.random(20.0)
        speed = (7.0 + self.random(3.0)) * 30.0
        self.set_speed(speed, angle)
