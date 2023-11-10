#!/bin/bash

# Start the API gateway
./target/release/api_gateway &

# Start Raft nodes (assuming 3 nodes for this example)
./target/release/raft_node --config=config_node_1.toml &
./target/release/raft_node --config=config_node_2.toml &
./target/release/raft_node --config=config_node_3.toml &

# Wait for all processes to complete
wait