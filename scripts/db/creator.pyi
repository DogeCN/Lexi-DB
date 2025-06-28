class Entry:
    phonetic: str
    definition: str
    translation: str
    exchanges: list[str]

    def __init__(
        self,
        phonetic: str,
        definition: str,
        translation: str,
        exchanges: list[str],
    ): ...

class Creator:
    def __init__(self, path: str, name: str, name_zh: str): ...
    def insert(self, key: str, value: Entry): ...
    def export(self): ...
