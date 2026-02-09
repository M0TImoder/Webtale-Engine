import math
import random

def init():
    return {
        "textureWait": "texture/enemy/spr_frogbullet_stop.png",
        "box": [217.0, 125.0, 417.0, 385.0],
        "rustSim": {
            "update": [
                ["timer", "timer - dt"],
                ["did_jump", "if(state == 0.0 && timer <= 0.0, 1.0, 0.0)"],
                ["state", "if(did_jump == 1.0, 1.0, state)"],
                ["vx", "if(did_jump == 1.0, jump_speed * cos(jump_angle * pi / 180.0), if(state == 1.0, vx + gx * dt, vx))"],
                ["vy", "if(did_jump == 1.0, jump_speed * sin(jump_angle * pi / 180.0), if(state == 1.0, vy + gy * dt, vy))"],
                ["texture", "if(did_jump == 1.0, \"texture/enemy/spr_frogbullet_go.png\", texture)"],
                ["x", "x + vx * dt"],
                ["y", "y + vy * dt"]
            ],
            "delete": "y < -300.0"
        }
    }

def spawn():
    grav_speed = 0.4 * 30.0 * 30.0
    gx = grav_speed * math.cos(math.radians(280.0))
    gy = grav_speed * math.sin(math.radians(280.0))
    return {
        "vars": {
            "state": 0.0,
            "timer": 0.5 + random.random() * 0.5,
            "did_jump": 0.0,
            "vx": 0.0,
            "vy": 0.0,
            "gx": gx,
            "gy": gy,
            "jump_speed": (7.0 + random.random() * 3.0) * 30.0,
            "jump_angle": 145.0 - random.random() * 20.0
        },
        "damage": 4
    }
