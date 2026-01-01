class Entry:
    phonetic: str
    definition: str
    translation: str
    exchanges: list[str]

class Handle:
    name: str
    name_zh: str

    def switch(self): ...
    def __len__(self) -> int: ...

class Manager:
    def __init__(self): ...
    def create(self, path: str, temp: str, callback: function) -> Handle:
        """
        Callback state:
            Unloaded => 0,
            Loading => 1,
            Loaded:
                Disabled => 2,
                Enabled => 3,
            Error => 4,
        """
        ...

    def get(self, word: str) -> Entry | None: ...
    def find(self, target: str) -> str | None: ...
    def filter(self, word: str, seps: list[str]) -> list[str]: ...
    def random(self) -> str | None: ...
    def clear(self): ...
