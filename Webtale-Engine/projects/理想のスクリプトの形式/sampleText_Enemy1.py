@text
class FroggitText:
    # ACT一覧の定義
    act = ["Check", "Compliment", "Threaten"]

    # 静的なテキストはクラス変数として定義
    actText_Check = "* FROGGIT - ATK 4 DEF 5\n* Life is difficult for this enemy."
    actText_Compliment = "* Froggit didn't understand what you said,\n  but was flattered anyway."
    actText_Threaten = "* Froggit didn't understand what you said,\n  but was scared anyway."

    # 条件によって変わるテキストは「@property」を使用する
    # これにより、システム側から参照された瞬間に計算が行われる
    @property
    def bubbleText(self):
        # Python 3.10の match-case 構文を使用
        # gameオブジェクトがグローバル、あるいは引数で渡される想定
        match game.turnCount:
            case 1:
                return "turn1"
            case 2:
                return "turn2"
            case 3:
                return "turn3"
            case _:
                return "error"

    # さらなる応用：特定のフラグ状況でACTテキストを変える例
    @property
    def dynamicActText(self):
        if game.is_genocide_route:
            return "* They are shivering."
        return self.actText_Check
    