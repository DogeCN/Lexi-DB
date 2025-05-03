from db.creator import PyDBCreator, PyEntry
from db.reader import PyDBReader

value = PyEntry("phonetic", "definition", "translation", ["exchange1", "exchanges2"])

creator = PyDBCreator("test.db")
creator.insert("test1", value)
creator.export()

reader = PyDBReader("test.db", "test.values")
print(reader["test1"].exchanges)
