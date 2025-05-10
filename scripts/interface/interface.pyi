class Serializer:
    def from_string(value: str) -> bytes:
        """Serialize a string to bytes"""

    def from_uint(value: int) -> bytes:
        """Serialize an unsigned integer to bytes"""

    def from_string_list(values: list[str]) -> bytes:
        """Serialize a list of strings to bytes"""

    def from_uint_list(values: list[int]) -> bytes:
        """Serialize a list of unsigned integers to bytes"""

class Deserializer:
    def to_string(data: bytes) -> str:
        """Deserialize bytes to a string"""

    def to_uint(data: bytes) -> int:
        """Deserialize bytes to an unsigned integer"""

    def to_string_list(data: bytes) -> list[str]:
        """Deserialize bytes to a list of strings"""

    def to_uint_list(data: bytes) -> list[int]:
        """Deserialize bytes to a list of unsigned integers"""

class Compressor:
    def compress(data: bytes) -> bytes:
        """Compress bytes"""

    def decompress(data: bytes) -> bytes:
        """Decompress bytes"""
