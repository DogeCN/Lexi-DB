from interface.interface import *


def test_string():
    print("\n===== 测试字符串序列化/反序列化 =====")

    test_str = "Hello, Lexi-DB!"
    print(f"原始字符串: {test_str}")

    # 序列化
    serialized = serialize_string(test_str)
    print(f"序列化后 (bytes): {serialized}")

    # 反序列化
    deserialized = deserialize_string(serialized)
    print(f"反序列化后: {deserialized}")
    print(f"结果匹配: {test_str == deserialized}")


def test_uint():
    print("\n===== 测试整数序列化/反序列化 =====")

    test_num = 12345
    print(f"原始整数: {test_num}")

    # 序列化
    serialized = serialize_uint(test_num)
    print(f"序列化后 (bytes): {serialized}")

    # 反序列化
    deserialized = deserialize_uint(serialized)
    print(f"反序列化后: {deserialized}")
    print(f"结果匹配: {test_num == deserialized}")


def test_string_list():
    print("\n===== 测试字符串列表序列化/反序列化 =====")

    test_list = ["apple", "banana", "cherry", "date"]
    print(f"原始列表: {test_list}")

    # 序列化
    serialized = serialize_string_list(test_list)
    print(f"序列化后 (bytes): {serialized}")

    # 反序列化
    deserialized = deserialize_string_list(serialized)
    print(f"反序列化后: {deserialized}")
    print(f"结果匹配: {test_list == deserialized}")


def test_uint_list():
    print("\n===== 测试整数列表序列化/反序列化 =====")

    test_list = [1, 2, 3, 4, 5, 1000, 10000, 100000]
    print(f"原始列表: {test_list}")

    # 序列化
    serialized = serialize_uint_list(test_list)
    print(f"序列化后 (bytes): {serialized}")

    # 反序列化
    deserialized = deserialize_uint_list(serialized)
    print(f"反序列化后: {deserialized}")
    print(f"结果匹配: {test_list == deserialized}")


def test_performance():
    print("\n===== 性能测试 =====")
    import time

    # 创建大型测试数据
    large_string = "x" * 100000
    large_list = list(range(10000))

    # 测试字符串序列化性能
    start_time = time.time()
    serialized_str = serialize_string(large_string)
    serialize_time = time.time() - start_time
    print(f"序列化大字符串 (100,000字符) 耗时: {serialize_time:.6f}秒")

    # 测试字符串反序列化性能
    start_time = time.time()
    deserialized_str = deserialize_string(serialized_str)
    deserialize_time = time.time() - start_time
    print(f"反序列化大字符串耗时: {deserialize_time:.6f}秒")
    print(f"字符串匹配: {large_string == deserialized_str}")

    # 测试列表序列化性能
    start_time = time.time()
    serialized_list = serialize_uint_list(large_list)
    serialize_time = time.time() - start_time
    print(f"序列化大整数列表 (10,000项) 耗时: {serialize_time:.6f}秒")

    # 测试列表反序列化性能
    start_time = time.time()
    deserialized_list = deserialize_uint_list(serialized_list)
    deserialize_time = time.time() - start_time
    print(f"反序列化大整数列表耗时: {deserialize_time:.6f}秒")
    print(f"列表匹配: {large_list == deserialized_list}")


if __name__ == "__main__":
    print("开始测试优化后的Lexi-DB序列化/反序列化接口")

    try:
        test_string()
        test_uint()
        test_string_list()
        test_uint_list()
        test_performance()
        print("\n所有测试完成!")
    except Exception as e:
        print(f"测试过程中出错: {e}")
        import traceback

        traceback.print_exc()
