import phase_api as phase

def phase_script(cls):
    cls.__is_phase__ = True
    return cls

@phase_script
class PhaseExample:
    def update(self, context):
        turn = phase.getTurn()
        if turn is None:
            turn = 0
        phase.setDialogText("* Turn {}".format(turn))
        phase.setBubbleMessage("Turn {}".format(turn))
        phase.setBubbleTexture("blconsm")
        phase.setBubblePosition(360.0, 65.0)
        return phase.getState()

_phase = PhaseExample()

def update(context):
    return _phase.update(context)
