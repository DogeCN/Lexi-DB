from lexi_db.creator import PyDBCreator, PyEntry
from lexi_db.reader import PyDBReader

value = PyEntry("phonetic", "definition", "translation", ["exchange1", "exchanges2"])

creator = PyDBCreator("test.db")
creator.insert("test1", value)
creator.export()

reader = PyDBReader("test.db")
print(reader["test1"].exchanges)
