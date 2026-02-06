import phase_api as phase

def update(context):
    turn = phase.getTurn()
    if turn is None:
        turn = 0
    phase.setDialogText("* Turn {}".format(turn))
    phase.setBubbleMessage("Turn {}".format(turn))
    phase.setBubbleTexture("blconsm")
    phase.setBubblePosition(360.0, 65.0)
    return phase.getState()
