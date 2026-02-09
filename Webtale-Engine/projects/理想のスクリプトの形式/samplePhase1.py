@phase
class BattlePhase1:
    def update(self, context):
        # ターン数や敵のHPに応じて弾幕を切り替える
        match context.turn:
            case 1 | 2:
                # 序盤は緩い攻撃
                return attack.select("attackA")
            case 3:
                # 3ターン目にセリフとともに攻撃を変える
                enemy.set_bubble("Here it comes!")
                return attack.select("attackB")
            case _:
                # HPが半分以下なら発狂モードへ
                if enemy.hp < (enemy.max_hp / 2):
                    return phase.next("SeriousPhase")
                return attack.select("attackA")

    # フェーズ開始時の処理
    def on_enter(self):
        enemy.set_dialog("* Froggit looks intent on fighting.")
        