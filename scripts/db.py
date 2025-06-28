from db.creator import Creator, Entry
from db.reader import Manager
from os import remove

value = Entry("phonetic", "definition", "translation", ["exchange1", "exchanges2"])

creator = Creator("test.db", "Name", "名称")
for i in range(100):
    creator.insert(f"test{i}", value)
creator.export()

manager = Manager()
handle = manager.create("test.db", "test.values", print)
handle.switch()
handle.switch()
assert handle["test1"].exchanges == ["exchange1", "exchanges2"]
assert handle["test1"].phonetic == "phonetic"
assert handle["test1"].definition == "definition"
assert handle["test1"].translation == "translation"
assert handle.name == "Name"
assert handle.name_zh == "名称"

handle.switch()
assert manager.find("tost99") == "test99"
assert manager.find("notfound") is None

remove("test.db")
print("All tests passed!")
