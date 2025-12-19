import math
import random

class Bullet:
    def __init__(self):
        self.x = 0.0
        self.y = 0.0
        self.vx = 0.0
        self.vy = 0.0
        self.texture = None
        self.shouldDelete = False
        
        self.state = "start"
        self.timer = 0.0

    def sysUpdate(self, dt):
        self.update(dt)
        self.x += self.vx * dt
        self.y += self.vy * dt

    def update(self, dt):
        pass

    def delete(self):
        self.shouldDelete = True

    def setTexture(self, name):
        self.texture = name

    def setSpeed(self, speed, angleDeg):
        rad = math.radians(angleDeg)
        self.vx = speed * math.cos(rad)
        self.vy = speed * math.sin(rad)

    def addVelocity(self, speed, angleDeg):
        rad = math.radians(angleDeg)
        self.vx += speed * math.cos(rad)
        self.vy += speed * math.sin(rad)

    def setPos(self, x, y):
        self.x = x
        self.y = y

    def random(self, minVal, maxVal=None):
        if maxVal is None:
            return random.uniform(0, minVal)
        return random.uniform(minVal, maxVal)

    def chasePlayer(self, playerX, playerY, speed):
        dx = playerX - self.x
        dy = playerY - self.y
        rad = math.atan2(dy, dx)
        self.vx = speed * math.cos(rad)
        self.vy = speed * math.sin(rad)
        