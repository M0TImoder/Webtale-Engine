import math
import random

class Bullet:
    def __init__(self):
        self.x = 0.0
        self.y = 0.0
        self.vx = 0.0
        self.vy = 0.0
        self.texture = None
        self.should_delete = False
        
        # 簡易的なステート管理用
        self.state = "start"
        self.timer = 0.0

    def sys_update(self, dt):
        """システムから呼ばれる更新処理"""
        self.update(dt)
        self.x += self.vx * dt
        self.y += self.vy * dt

    def update(self, dt):
        """ユーザーがオーバーライドする更新処理"""
        pass

    def delete(self):
        """弾を削除する"""
        self.should_delete = True

    def set_texture(self, name):
        """テクスチャを変更する"""
        self.texture = name

    def set_speed(self, speed, angle_deg):
        """速度と角度(度)を設定する"""
        rad = math.radians(angle_deg)
        self.vx = speed * math.cos(rad)
        self.vy = speed * math.sin(rad)

    def add_velocity(self, speed, angle_deg):
        """現在の速度にベクトルを加算する"""
        rad = math.radians(angle_deg)
        self.vx += speed * math.cos(rad)
        self.vy += speed * math.sin(rad)

    def set_pos(self, x, y):
        self.x = x
        self.y = y

    # --- 便利なユーティリティ ---
    def random(self, min_val, max_val=None):
        """random(max) または random(min, max)"""
        if max_val is None:
            return random.uniform(0, min_val)
        return random.uniform(min_val, max_val)

    def chase_player(self, player_x, player_y, speed):
        """プレイヤーの方へ速度を設定する"""
        dx = player_x - self.x
        dy = player_y - self.y
        rad = math.atan2(dy, dx)
        self.vx = speed * math.cos(rad)
        self.vy = speed * math.sin(rad)
