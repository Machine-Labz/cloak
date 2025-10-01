#!/bin/bash

# Example API usage for the relay service
# This script demonstrates how to submit a withdraw request and check its status

BASE_URL="http://localhost:3002"

echo "=== Cloak Relay Service API Example ==="
echo "Base URL: $BASE_URL"
echo

# 1. Health check
echo "1. Health Check"
echo "GET $BASE_URL/health"
curl -s "$BASE_URL/health" | jq '.' 2>/dev/null || curl -s "$BASE_URL/health"
echo -e "\n"

# 2. Submit a withdraw request
echo "2. Submit Withdraw Request"
echo "POST $BASE_URL/withdraw"

# Create test data (this would come from the ZK proof in real usage)
WITHDRAW_REQUEST='{
  "outputs": [
    {
      "recipient": "11111111111111111111111111111112",
      "amount": 990000
    }
  ],
  "policy": {
    "fee_bps": 100
  },
  "public_inputs": {
    "root": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "nf": "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
    "amount": 1000000,
    "fee_bps": 100,
    "outputs_hash": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
  },
  "proof_bytes": "'$(base64 -w 0 <<< $(yes '\x00' | head -c 256))'"
}'

echo "Request body:"
echo "$WITHDRAW_REQUEST" | jq '.' 2>/dev/null || echo "$WITHDRAW_REQUEST"
echo

# Submit the request
RESPONSE=$(curl -s -X POST "$BASE_URL/withdraw" \
  -H "Content-Type: application/json" \
  -d "$WITHDRAW_REQUEST")

echo "Response:"
echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"

# Extract request ID for status check
REQUEST_ID=$(echo "$RESPONSE" | jq -r '.data.request_id' 2>/dev/null)

if [ "$REQUEST_ID" != "null" ] && [ "$REQUEST_ID" != "" ]; then
    echo -e "\nRequest ID: $REQUEST_ID"
    
    # 3. Check status
    echo -e "\n3. Check Status"
    echo "GET $BASE_URL/status/$REQUEST_ID"
    
    # Wait a moment
    sleep 1
    
    STATUS_RESPONSE=$(curl -s "$BASE_URL/status/$REQUEST_ID")
    echo "Response:"
    echo "$STATUS_RESPONSE" | jq '.' 2>/dev/null || echo "$STATUS_RESPONSE"
else
    echo "Failed to extract request ID from response"
fi

echo -e "\n=== Example Complete ==="
echo
echo "Note: This example uses test data. In production:"
echo "- proof_bytes would be a real SP1 proof (256 bytes)"
echo "- public_inputs would be extracted from the proof"
echo "- recipient addresses would be valid Solana pubkeys"
echo "- The service would need PostgreSQL and Redis running" 