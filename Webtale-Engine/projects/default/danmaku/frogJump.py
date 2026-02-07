from api import Bullet

def init():
    return {
        "textureWait": "texture/enemy/spr_frogbullet_stop.png",
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
        
        gravSpeed = 0.4 * 30.0 * 30.0
        self.gx, self.gy = 0, 0
        
        import math
        self.gx = gravSpeed * math.cos(math.radians(280.0))
        self.gy = gravSpeed * math.sin(math.radians(280.0))

    def update(self, dt):
        if self.state == "waiting":
            self.timer -= dt
            if self.timer <= 0:
                self.jump()
                
        elif self.state == "jumping":
            self.vx += self.gx * dt
            self.vy += self.gy * dt

    def jump(self):
        self.state = "jumping"
        self.setTexture("texture/enemy/spr_frogbullet_go.png")
        
        angle = 145.0 - self.random(20.0)
        speed = (7.0 + self.random(3.0)) * 30.0
        self.setSpeed(speed, angle)
