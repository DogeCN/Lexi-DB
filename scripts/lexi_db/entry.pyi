class PyEntry:
    phonetic: str
    definition: str
    translation: str
    exchanges: list[str]

    def __init__(
        phonetic: str,
        definition: str,
        translation: str,
        exchanges: list[str],
    ): ...
