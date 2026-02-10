def tachie(cls):
    cls.__is_tachie__ = True
    return cls

@tachie
class FroggitTachie:
    headSwaySpeed = 2.0
    headSwayAmplitude = 2.0

def getTachieData():
    return {
        "headSwaySpeed": FroggitTachie.headSwaySpeed,
        "headSwayAmplitude": FroggitTachie.headSwayAmplitude
    }
