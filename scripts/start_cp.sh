#!/bin/bash

# Default to localhost if no IP provided
IP=${1:-localhost}

# Function to make curl request and check status
make_request() {
    local url=$1
    local method=${2:-GET}
    local data=$3
    local headers=$4
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -o /dev/null -w "%{http_code}" "$url")
    else
        response=$(curl -s -o /dev/null -w "%{http_code}" \
            -X "$method" \
            "$url" \
            -H 'accept: */*' \
            -H 'Content-Type: application/json' \
            $headers \
            -d "$data")
    fi
    
    if [ "$response" != "200" ]; then
        echo "Error: Request to $url failed with status code $response"
        exit 1
    fi
    
    echo "Request to $url successful"
}

# POST request with JSON payload
make_request "http://$IP:5000/api/generatorConfigSetup" "POST" '{
    "generator_config": [
      {
        "address": "0x31136b0076a21d3d363502a135ad9b0e8c82ea14",
        "data": "sample payload data",
        "supported_markets": ["1"]
      }
    ],
    "runtime_config": {
      "ws_url": "wss://arbitrum-sepolia-rpc.publicnode.com",
      "http_url": "https://sepolia-rollup.arbitrum.io/rpc",
      "private_key": "0x1111111111111111111111111111111111111111111111111111111111111111",
      "proof_market_place": "0xC05d689B341d84900f0d0CE36f35aDAbfB57F68d",
      "generator_registry": "0x4743a2c7a96C9FBED8b7eAD980aD01822F9711Db",
      "start_block": 130112284,
      "chain_id": 421614,
      "payment_token": "0x8230d71d809718132C2054704F5E3aF1b86B669C",
      "staking_token": "0xB5570D4D39dD20F61dEf7C0d6846790360b89a18",
      "attestation_verifier": "0xB5570D4D39dD20F61dEf7C0d6846790360b89a18",
      "entity_registry": "0x457D42573096b339bA48Be576e9Db4Fc5F186091",
      "markets": {
        "1": {
          "port": "8080",
          "ivs_url": "http://localhost:8080"
        }
      }
    }
}'

make_request "http://$IP:5000/api/startProgram" "POST" '{
  "program_name": "confidential_prover"
}'

echo "All requests completed successfully"
