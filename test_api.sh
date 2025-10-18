#!/bin/bash

# Test API endpoints for Project Kit

BASE_URL="http://localhost:3000"

echo "=== Testing Project Kit API ==="
echo

# Test root endpoint
echo "1. Testing root endpoint..."
curl -s "$BASE_URL/"
echo
echo

# Test signup
echo "2. Testing signup..."
SIGNUP_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/signup" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }')
echo "$SIGNUP_RESPONSE" | jq '.'
TOKEN=$(echo "$SIGNUP_RESPONSE" | jq -r '.token')
echo "Token: $TOKEN"
echo
echo

# Test login
echo "3. Testing login..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }')
echo "$LOGIN_RESPONSE" | jq '.'
echo
echo

# Test GET /db/users
echo "4. Testing GET /db/users..."
curl -s "$BASE_URL/db/users" | jq '.'
echo
echo

# Test POST /db/posts (if posts table exists)
echo "5. Testing POST /db/posts..."
curl -s -X POST "$BASE_URL/db/posts" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My First Post",
    "content": "Hello World",
    "user_id": 1,
    "created_at": "2024-10-18T00:00:00Z",
    "updated_at": "2024-10-18T00:00:00Z"
  }' | jq '.'
echo
echo

echo "=== Tests Complete ==="
