import math
import random

def danmaku_api(cls):
    cls.__is_danmaku_api__ = True
    return cls

_PLAYER_POS = (0.0, 0.0)
_PLAYER_VALID = False
_GLOBAL_RNG = None

class RNG:
    def __init__(self, seed=1):
        self._state = int(seed) & 0xFFFFFFFF

    def seed(self, seed):
        self._state = int(seed) & 0xFFFFFFFF

    def rand(self):
        self._state = (1103515245 * self._state + 12345) & 0x7FFFFFFF
        return self._state / 0x7FFFFFFF

    def uniform(self, min_val=0.0, max_val=1.0):
        return min_val + (max_val - min_val) * self.rand()

    def randint(self, min_val, max_val):
        return int(self.uniform(min_val, max_val + 1))

    def choice(self, seq):
        if not seq:
            return None
        return seq[self.randint(0, len(seq) - 1)]

def seed(value):
    global _GLOBAL_RNG
    _GLOBAL_RNG = RNG(value)

def _resolve_rng(rng=None, seed_value=None):
    if rng is not None:
        return rng
    if seed_value is not None:
        return RNG(seed_value)
    return _GLOBAL_RNG

def rand(min_val=0.0, max_val=None, rng=None, seed_value=None):
    if max_val is None:
        max_val = min_val
        min_val = 0.0
    rng = _resolve_rng(rng, seed_value)
    if rng is None:
        return random.uniform(min_val, max_val)
    return rng.uniform(min_val, max_val)

def choice(seq, rng=None, seed_value=None):
    rng = _resolve_rng(rng, seed_value)
    if not seq:
        return None
    if rng is None:
        idx = int(random.uniform(0, len(seq)))
        idx = clamp(idx, 0, len(seq) - 1)
        return seq[idx]
    return rng.choice(seq)

def set_player_position(x, y):
    global _PLAYER_POS, _PLAYER_VALID
    _PLAYER_POS = (float(x), float(y))
    _PLAYER_VALID = True

def get_player_position(default=None):
    if _PLAYER_VALID:
        return _PLAYER_POS
    if default is None:
        return (0.0, 0.0)
    return default

def player_pos(default=None):
    return get_player_position(default)

def player_x(default=0.0):
    return get_player_position((default, default))[0]

def player_y(default=0.0):
    return get_player_position((default, default))[1]

def clamp(value, min_val, max_val):
    return max(min_val, min(value, max_val))

def vec(x, y):
    return (float(x), float(y))

def add(a, b):
    return (a[0] + b[0], a[1] + b[1])

def sub(a, b):
    return (a[0] - b[0], a[1] - b[1])

def scale(v, s):
    return (v[0] * s, v[1] * s)

def length(v):
    return math.hypot(v[0], v[1])

def normalize(v):
    l = length(v)
    if l == 0.0:
        return (0.0, 0.0)
    return (v[0] / l, v[1] / l)

def angle_deg(v):
    return math.degrees(math.atan2(v[1], v[0]))

def angle_to(from_pos, to_pos):
    return angle_deg((to_pos[0] - from_pos[0], to_pos[1] - from_pos[1]))

def from_angle(speed, angle_deg_val):
    rad = math.radians(angle_deg_val)
    return (speed * math.cos(rad), speed * math.sin(rad))

def rotate(v, angle_deg_val):
    rad = math.radians(angle_deg_val)
    cos_v = math.cos(rad)
    sin_v = math.sin(rad)
    return (v[0] * cos_v - v[1] * sin_v, v[0] * sin_v + v[1] * cos_v)

def wrap_angle(angle_deg_val):
    while angle_deg_val > 180.0:
        angle_deg_val -= 360.0
    while angle_deg_val < -180.0:
        angle_deg_val += 360.0
    return angle_deg_val

def turn_towards(current_deg, target_deg, max_delta_deg):
    delta = wrap_angle(target_deg - current_deg)
    delta = clamp(delta, -max_delta_deg, max_delta_deg)
    return current_deg + delta

def ring_angles(count, start_deg=0.0):
    if count <= 0:
        return []
    step = 360.0 / count
    return [start_deg + (step * i) for i in range(count)]

def fan_angles(count, center_deg, spread_deg):
    if count <= 0:
        return []
    if count == 1:
        return [center_deg]
    start = center_deg - (spread_deg / 2.0)
    step = spread_deg / (count - 1)
    return [start + (step * i) for i in range(count)]

def random_angles(count, center_deg, spread_deg, rng=None, seed_value=None):
    if count <= 0:
        return []
    half = spread_deg / 2.0
    return [center_deg + rand(-half, half, rng=rng, seed_value=seed_value) for _ in range(count)]

def spiral_angles(count, start_deg, step_deg):
    if count <= 0:
        return []
    return [start_deg + (step_deg * i) for i in range(count)]

def ring_velocities(count, speed, start_deg=0.0):
    return [from_angle(speed, angle) for angle in ring_angles(count, start_deg)]

def fan_velocities(count, speed, center_deg, spread_deg):
    return [from_angle(speed, angle) for angle in fan_angles(count, center_deg, spread_deg)]

def random_velocities(count, speed, center_deg, spread_deg, rng=None, seed_value=None):
    return [from_angle(speed, angle) for angle in random_angles(count, center_deg, spread_deg, rng=rng, seed_value=seed_value)]

@danmaku_api
class Bullet:
    def __init__(self):
        self.x = 0.0
        self.y = 0.0
        self.vx = 0.0
        self.vy = 0.0
        self.texture = None
        self.shouldDelete = False
        self.damage = 0
        
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

    def random(self, minVal, maxVal=None, rng=None, seed_value=None):
        return rand(minVal, maxVal, rng=rng, seed_value=seed_value)

    def chasePlayer(self, playerX, playerY, speed):
        dx = playerX - self.x
        dy = playerY - self.y
        rad = math.atan2(dy, dx)
        self.vx = speed * math.cos(rad)
        self.vy = speed * math.sin(rad)

    def aimAt(self, targetX, targetY, speed, offsetDeg=0.0):
        angle = angle_to((self.x, self.y), (targetX, targetY)) + offsetDeg
        self.setSpeed(speed, angle)

    def aimAtPlayer(self, speed, offsetDeg=0.0):
        px, py = get_player_position()
        self.aimAt(px, py, speed, offsetDeg)

    def arcTo(self, targetX, targetY, speed, arcDeg):
        self.aimAt(targetX, targetY, speed, arcDeg)

    def arcToPlayer(self, speed, arcDeg):
        px, py = get_player_position()
        self.arcTo(px, py, speed, arcDeg)

    def curveTo(self, targetX, targetY, speed=None, maxTurnDeg=5.0):
        target_angle = angle_to((self.x, self.y), (targetX, targetY))
        if speed is None:
            speed = math.hypot(self.vx, self.vy)
        if abs(self.vx) < 1e-6 and abs(self.vy) < 1e-6:
            current_angle = target_angle
        else:
            current_angle = angle_deg((self.vx, self.vy))
        next_angle = turn_towards(current_angle, target_angle, maxTurnDeg)
        self.setSpeed(speed, next_angle)

    def curveToPlayer(self, speed=None, maxTurnDeg=5.0):
        px, py = get_player_position()
        self.curveTo(px, py, speed, maxTurnDeg)
