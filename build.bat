@echo off
set pyd1=scripts\db\reader.pyd
set pyd2=scripts\db\creator.pyd
cargo build --release
copy >nul /y target\release\reader.dll %pyd1%
copy >nul /y target\release\creator.dll %pyd2%
