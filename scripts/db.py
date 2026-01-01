from db.creator import Creator, Entry
from db.reader import Manager
from os import remove
from threading import Thread

value = Entry("phonetic", "definition", "translation", ["exchange1", "exchanges2"])

creator = Creator("test.db", "Name", "名称")
for i in range(100):
    creator.insert(f"test{i}", value)
creator.export()

manager = Manager()
handle = manager.create("test.db", "test.values", print)
print(handle.name, handle.name_zh)
handle.switch()
test1 = manager.get("test1")
assert test1.exchanges == ["exchange1", "exchanges2"]
assert test1.phonetic == "phonetic"
assert test1.definition == "definition"
assert test1.translation == "translation"
handle.switch()
assert manager.get("test1") == None
assert handle.name == "Name"
assert handle.name_zh == "名称"

handle.switch()
assert manager.find("tost99") == "test99"
assert manager.find("notfound") is None

remove("test.db")
print("All tests passed!")
