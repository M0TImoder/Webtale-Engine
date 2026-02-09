@enemy
class Froggit:
    enemyName = "Froggit"
    enemyHp = 30
    enemyMaxHp = 30
    enemyAtk = 4
    enemyDef = 5
    
    # 文字列ではなくシンボルとして定義 (Center, Left 等)
    pos = "Center" 
    
    # 立ち絵定義クラスやスクリプト名を指定
    tachieScript = "Enemy1Tachie"

    # 動的なステータス変化の例 (HPが減ると攻撃力が上がるなど)
    @property
    def currentAtk(self):
        if self.enemyHp < (self.enemyMaxHp / 2):
            return self.enemyAtk + 2
        return self.enemyAtk
    