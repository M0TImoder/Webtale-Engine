def phase_api(cls):
    cls.__is_phase_api__ = True
    return cls

@phase_api
class PhaseApi:
    _state = {}
    _context = {}

    @classmethod
    def reset(cls, context):
        cls._context = context
        cls._state = {
            "dialogText": None,
            "attackPatterns": None,
            "bubbleMessages": None,
            "bubbleMessage": None,
            "bubbleTexture": None,
            "bubblePosition": None,
            "nextPhase": None,
        }

    @classmethod
    def getContext(cls):
        return cls._context

    @classmethod
    def getTurn(cls):
        return cls._context.get("turn")

    @classmethod
    def getLastPlayerAction(cls):
        return cls._context.get("lastPlayerAction")

    @classmethod
    def getLastActCommand(cls):
        return cls._context.get("lastActCommand")

    @classmethod
    def setDialogText(cls, text):
        cls._state["dialogText"] = text

    @classmethod
    def setAttackPatterns(cls, patterns):
        cls._state["attackPatterns"] = patterns

    @classmethod
    def setBubbleMessages(cls, messages):
        cls._state["bubbleMessages"] = messages

    @classmethod
    def setBubbleMessage(cls, message):
        cls._state["bubbleMessage"] = message

    @classmethod
    def setBubbleTexture(cls, path):
        cls._state["bubbleTexture"] = path

    @classmethod
    def setBubblePosition(cls, x, y):
        cls._state["bubblePosition"] = [x, y]

    @classmethod
    def setNextPhase(cls, name):
        cls._state["nextPhase"] = name

    @classmethod
    def getState(cls):
        return {key: value for key, value in cls._state.items() if value is not None}

    @classmethod
    def getInitialPhase(cls):
        return "phase1"

def reset(context):
    PhaseApi.reset(context)

def getContext():
    return PhaseApi.getContext()

def getTurn():
    return PhaseApi.getTurn()

def getLastPlayerAction():
    return PhaseApi.getLastPlayerAction()

def getLastActCommand():
    return PhaseApi.getLastActCommand()

def setDialogText(text):
    PhaseApi.setDialogText(text)

def setAttackPatterns(patterns):
    PhaseApi.setAttackPatterns(patterns)

def setBubbleMessages(messages):
    PhaseApi.setBubbleMessages(messages)

def setBubbleMessage(message):
    PhaseApi.setBubbleMessage(message)

def setBubbleTexture(path):
    PhaseApi.setBubbleTexture(path)

def setBubblePosition(x, y):
    PhaseApi.setBubblePosition(x, y)

def setNextPhase(name):
    PhaseApi.setNextPhase(name)

def getState():
    return PhaseApi.getState()

def getInitialPhase():
    return PhaseApi.getInitialPhase()
