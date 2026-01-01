@echo off
cargo test --release
cargo build --release
copy >nul /y target\release\reader.dll scripts\db\reader.pyd
copy >nul /y target\release\creator.dll scripts\db\creator.pyd
