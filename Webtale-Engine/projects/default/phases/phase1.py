import phase_api as phase

def phase(cls):
    cls.__is_phase__ = True
    return cls

@phase
class Phase1:
    def update(self, context):
        return phase.getState()

_phase = Phase1()

def update(context):
    return _phase.update(context)
