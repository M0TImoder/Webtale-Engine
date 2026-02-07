_state = {}
_context = {}

def reset(context):
    global _context, _state
    _context = context
    _state = {
        "dialogText": None,
        "attackPatterns": None,
        "bubbleMessages": None,
        "bubbleMessage": None,
        "bubbleTexture": None,
        "bubblePosition": None,
        "nextPhase": None,
    }

def getContext():
    return _context

def getTurn():
    return _context.get("turn")

def getLastPlayerAction():
    return _context.get("lastPlayerAction")

def getLastActCommand():
    return _context.get("lastActCommand")

def setDialogText(text):
    _state["dialogText"] = text

def setAttackPatterns(patterns):
    _state["attackPatterns"] = patterns

def setBubbleMessages(messages):
    _state["bubbleMessages"] = messages

def setBubbleMessage(message):
    _state["bubbleMessage"] = message

def setBubbleTexture(path):
    _state["bubbleTexture"] = path

def setBubblePosition(x, y):
    _state["bubblePosition"] = [x, y]

def setNextPhase(name):
    _state["nextPhase"] = name

def getState():
    return _state

def getInitialPhase():
    return "phase1"
