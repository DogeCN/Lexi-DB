from db.creator import PyDBCreator, PyEntry
from db.reader import PyDBReader
from os import remove

value = PyEntry("phonetic", "definition", "translation", ["exchange1", "exchanges2"])

creator = PyDBCreator("test.db")
for i in range(2):
    creator.insert(f"test{i}", value)
creator.export()

reader = PyDBReader("test.db", "test.values")
assert reader["test1"].exchanges == ["exchange1", "exchanges2"]
assert reader["test1"].phonetic == "phonetic"
assert reader["test1"].definition == "definition"
assert reader["test1"].translation == "translation"
assert set(reader.keys()) == {"test0", "test1"}

remove("test.db")

print("All tests passed!")
