def serialize_string(value: str) -> bytearray:
    """Serialize a string to bytes"""

def serialize_uint(value: int) -> bytearray:
    """Serialize an unsigned integer to bytes"""

def serialize_string_list(values: list[str]) -> bytearray:
    """Serialize a list of strings to bytes"""

def serialize_uint_list(values: list[int]) -> bytearray:
    """Serialize a list of unsigned integers to bytes"""

def deserialize_string(data: bytes) -> str:
    """Deserialize bytes to a string"""

def deserialize_uint(data: bytes) -> int:
    """Deserialize bytes to an unsigned integer"""

def deserialize_string_list(data: bytes) -> list[str]:
    """Deserialize bytes to a list of strings"""

def deserialize_uint_list(data: bytes) -> list[int]:
    """Deserialize bytes to a list of unsigned integers"""
