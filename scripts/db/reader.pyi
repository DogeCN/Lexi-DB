class PyEntry:
    matched: bool
    phonetic: str
    definition: str
    translation: str
    exchanges: list[str]

class PyDBReader:
    name: str
    name_zh: str
    def __init__(self, path: str, temp: str): ...
    def load(self) -> None: ...
    def filter(self, word: str, seps: list[str]) -> list[PyEntry]: ...
    def __getitem__(self, key: str) -> PyEntry | None: ...
    def __len__(self) -> int: ...
    def __contains__(self, key: str) -> bool: ...
