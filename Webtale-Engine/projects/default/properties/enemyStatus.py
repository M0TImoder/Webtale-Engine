def enemy(cls):
    cls.__is_enemy__ = True
    return cls

@enemy
class Froggit:
    enemyName = "Froggit"
    enemyHp = 30
    enemyMaxHp = 30
    enemyAtk = 4
    enemyDef = 5
    dialogText = "* Froggit hops close!"
    phaseScript = "PhaseExample"
    attackPatterns = ["frogJump"]
    actCommands = ["Check", "Compliment", "Threaten"]
    actTexts = {
        "Check": "* FROGGIT - ATK 4 DEF 5\n* Life is difficult for this enemy.",
        "Compliment": "* Froggit didn't understand what you said,\n  but was flattered anyway.",
        "Threaten": "* Froggit didn't understand what you said,\n  but was scared anyway."
    }
    bubbleMessages = ["Ribbit, ribbit.", "Croak.", "Hop, hop."]
    bodyTexture = "texture/enemy/spr_froglegs_0.png"
    headTexture = "texture/enemy/spr_froghead_0.png"
    headYOffset = 22.0
    tachieScript = "froggit"
    baseX = 320.0
    baseY = 160.0
    scale = 1.0

def getEnemyStatus():
    return {
        "enemyName": Froggit.enemyName,
        "enemyHp": Froggit.enemyHp,
        "enemyMaxHp": Froggit.enemyMaxHp,
        "enemyAtk": Froggit.enemyAtk,
        "enemyDef": Froggit.enemyDef,
        "dialogText": Froggit.dialogText,
        "phaseScript": Froggit.phaseScript,
        "attackPatterns": Froggit.attackPatterns,
        "actCommands": Froggit.actCommands,
        "actTexts": Froggit.actTexts,
        "bubbleMessages": Froggit.bubbleMessages,
        "bodyTexture": Froggit.bodyTexture,
        "headTexture": Froggit.headTexture,
        "headYOffset": Froggit.headYOffset,
        "tachieScript": Froggit.tachieScript,
        "baseX": Froggit.baseX,
        "baseY": Froggit.baseY,
        "scale": Froggit.scale
    }
