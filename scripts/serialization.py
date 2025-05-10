from interface.interface import *


def test_string():
    print("\n===== 测试字符串序列化/反序列化 =====")

    test_str = "Hello, Lexi-DB!"
    print(f"原始字符串: {test_str}")

    # 序列化
    serialized = Serializer.from_string(test_str)
    print(f"序列化后 (bytes): {serialized}")

    # 反序列化
    deserialized = Deserializer.to_string(serialized)
    print(f"反序列化后: {deserialized}")
    print(f"结果匹配: {test_str == deserialized}")


def test_uint():
    print("\n===== 测试整数序列化/反序列化 =====")

    test_num = 12345
    print(f"原始整数: {test_num}")

    # 序列化
    serialized = Serializer.from_uint(test_num)
    print(f"序列化后 (bytes): {serialized}")

    # 反序列化
    deserialized = Deserializer.to_uint(serialized)
    print(f"反序列化后: {deserialized}")
    print(f"结果匹配: {test_num == deserialized}")


def test_string_list():
    print("\n===== 测试字符串列表序列化/反序列化 =====")

    test_list = ["apple", "banana", "cherry", "date"]
    print(f"原始列表: {test_list}")

    # 序列化
    serialized = Serializer.from_string_list(test_list)
    print(f"序列化后 (bytes): {serialized}")

    # 反序列化
    deserialized = Deserializer.to_string_list(serialized)
    print(f"反序列化后: {deserialized}")
    print(f"结果匹配: {test_list == deserialized}")


def test_uint_list():
    print("\n===== 测试整数列表序列化/反序列化 =====")

    test_list = [1, 2, 3, 4, 5, 1000, 10000, 100000]
    print(f"原始列表: {test_list}")

    # 序列化
    serialized = Serializer.from_uint_list(test_list)
    print(f"序列化后 (bytes): {serialized}")

    # 反序列化
    deserialized = Deserializer.to_uint_list(serialized)
    print(f"反序列化后: {deserialized}")
    print(f"结果匹配: {test_list == deserialized}")


def test_compression():
    print("\n===== 测试数据压缩/解压缩 =====")

    # 创建测试数据 - 使用重复性高的数据以便于压缩
    test_data = b"abcdefghijklmnopqrstuvwxyz" * 100
    original_size = len(test_data)
    print(f"原始数据大小: {original_size} 字节")

    # 压缩
    compressed = Compressor.compress(test_data)
    compressed_size = len(compressed)
    print(f"压缩后大小: {compressed_size} 字节")
    print(f"压缩率: {compressed_size / original_size:.2%}")

    # 解压缩
    decompressed = Compressor.decompress(compressed)
    print(f"解压缩后大小: {len(decompressed)} 字节")
    print(f"数据匹配: {test_data == decompressed}")


def test_compression_with_serialization():
    print("\n===== 测试序列化+压缩组合 =====")

    # 创建测试数据
    test_list = ["测试数据"] * 1000  # 创建包含大量重复数据的列表
    print(f"原始列表长度: {len(test_list)} 项")

    # 序列化
    serialized = Serializer.from_string_list(test_list)
    serialized_size = len(serialized)
    print(f"序列化后大小: {serialized_size} 字节")

    # 压缩序列化数据
    compressed = Compressor.compress(serialized)
    compressed_size = len(compressed)
    print(f"压缩后大小: {compressed_size} 字节")
    print(f"压缩率: {compressed_size / serialized_size:.2%}")

    # 解压缩
    decompressed = Compressor.decompress(compressed)
    print(f"解压缩后大小: {len(decompressed)} 字节")
    print(f"解压缩数据与序列化数据匹配: {serialized == decompressed}")

    # 反序列化
    deserialized = Deserializer.to_string_list(decompressed)
    print(f"反序列化后列表长度: {len(deserialized)} 项")
    print(f"数据匹配: {test_list == deserialized}")


def test_performance():
    print("\n===== 性能测试 =====")
    import time

    # 创建大型测试数据
    large_string = "x" * 100000
    large_list = list(range(10000))

    # 测试字符串序列化性能
    start_time = time.time()
    serialized_str = Serializer.from_string(large_string)
    serialize_time = time.time() - start_time
    print(f"序列化大字符串 (100,000字符) 耗时: {serialize_time:.6f}秒")

    # 测试字符串反序列化性能
    start_time = time.time()
    deserialized_str = Deserializer.to_string(serialized_str)
    deserialize_time = time.time() - start_time
    print(f"反序列化大字符串耗时: {deserialize_time:.6f}秒")
    print(f"字符串匹配: {large_string == deserialized_str}")

    # 测试列表序列化性能
    start_time = time.time()
    serialized_list = Serializer.from_uint_list(large_list)
    serialize_time = time.time() - start_time
    print(f"序列化大整数列表 (10,000项) 耗时: {serialize_time:.6f}秒")

    # 测试列表反序列化性能
    start_time = time.time()
    deserialized_list = Deserializer.to_uint_list(serialized_list)
    deserialize_time = time.time() - start_time
    print(f"反序列化大整数列表耗时: {deserialize_time:.6f}秒")
    print(f"列表匹配: {large_list == deserialized_list}")

    # 测试压缩性能
    print("\n===== 压缩性能测试 =====")

    # 创建可压缩的大型数据
    compressible_data = b"abcdefghijklmnopqrstuvwxyz" * 10000

    # 测试压缩性能
    start_time = time.time()
    compressed = Compressor.compress(compressible_data)
    compress_time = time.time() - start_time
    print(f"压缩 {len(compressible_data)} 字节数据耗时: {compress_time:.6f}秒")
    print(f"压缩率: {len(compressed) / len(compressible_data):.2%}")

    # 测试解压缩性能
    start_time = time.time()
    decompressed = Compressor.decompress(compressed)
    decompress_time = time.time() - start_time
    print(f"解压缩 {len(compressed)} 字节数据耗时: {decompress_time:.6f}秒")
    print(f"数据匹配: {compressible_data == decompressed}")


if __name__ == "__main__":
    print("开始测试接口")

    try:
        test_string()
        test_uint()
        test_string_list()
        test_uint_list()
        test_compression()
        test_compression_with_serialization()
        test_performance()
        print("\n所有测试完成!")
    except Exception as e:
        print(f"测试过程中出错: {e}")
        import traceback

        traceback.print_exc()
