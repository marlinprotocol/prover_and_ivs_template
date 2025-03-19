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
      "address": "0xc6de583b87716e351e4fb60d687b9330877dbaf4",
      "data": "Some Data",
      "supported_markets": [
        "3"
      ]
    }
  ],
  "runtime_config": {
    "ws_url": "wss://arb-sepolia.g.alchemy.com/v2/somePlaceHolder",
    "http_url": "https://arb-sepolia.g.alchemy.com/v2/l86jFYjBFWZTQMRof96TpIGigjbZMUcr",
    "private_key": "c53dd8e14d0a4f8fa7b87c66adfc0d6197159732fd29517ea6783741423b9f54",
    "proof_market_place": "0xc05d689b341d84900f0d0ce36f35adabfb57f68d",
    "generator_registry": "0x4743a2c7a96c9fbed8b7ead980ad01822f9711db",
    "start_block": 115108807,
    "chain_id": 421614,
    "payment_token": "0x8230d71d809718132c2054704f5e3af1b86b669c",
    "staking_token": "0xb5570d4d39dd20f61def7c0d6846790360b89a18",
    "attestation_verifier": "0x63eef1576b477aa60bfd7300b2c85b887639ac1b",
    "entity_registry": "0x457d42573096b339ba48be576e9db4fc5f186091",
    "markets": {
      "3": {
        "port": "8080",
        "ivs_url": "http://localhost:8080/api/checkInput"
      }
    }
  }
}'

make_request "http://$IP:5000/api/startProgram" "POST" '{
  "program_name": "confidential_prover"
}'

echo "All requests completed successfully"
