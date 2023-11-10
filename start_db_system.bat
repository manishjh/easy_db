@echo off

REM build in release mode
cargo build --release

REM Start Raft nodes
start target\release\raft_node.exe --id=1 --count=3
start target\release\raft_node.exe --id=2 --count=3
start target\release\raft_node.exe --id=3 --count=3

REM Start the API gateway
start target\release\api_gateway.exe --id=0 --count=3

