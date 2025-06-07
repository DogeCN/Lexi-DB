from db.creator import PyDBCreator, PyEntry
from db.reader import PyDBReader
from os import remove

value = PyEntry("phonetic", "definition", "translation", ["exchange1", "exchanges2"])

creator = PyDBCreator("test.db", "Name", "名称")
for i in range(100):
    creator.insert(f"test{i}", value)
creator.export()

reader = PyDBReader("test.db", "test.values")
reader.load()
assert reader["test1"].exchanges == ["exchange1", "exchanges2"]
assert reader["test1"].phonetic == "phonetic"
assert reader["test1"].definition == "definition"
assert reader["test1"].translation == "translation"
assert reader["tost99"].matched == True
assert reader.name == "Name"
assert reader.name_zh == "名称"

remove("test.db")

print("All tests passed!")
