import phase_api as phase

def phase_script(cls):
    cls.__is_phase__ = True
    return cls

@phase_script
class Phase1:
    def update(self, context):
        return phase.getState()

_phase = Phase1()

def update(context):
    return _phase.update(context)
