@echo off
cargo build --release
copy >nul /y target\release\reader.dll scripts\db\reader.pyd
copy >nul /y target\release\creator.dll scripts\db\creator.pyd
copy >nul /y target\release\interface.dll scripts\interface\interface.pyd
