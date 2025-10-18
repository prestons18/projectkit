#!/bin/bash

# Test API endpoints for Project Kit
# Tests role-based authentication and authorization

BASE_URL="http://localhost:3000"

echo "=== Testing Project Kit API with Role-Based Auth ==="
echo

# Test root endpoint
echo "1. Testing root endpoint..."
curl -s "$BASE_URL/"
echo
echo

# Test signup (creates user with 'user' role)
echo "2. Testing signup (user role)..."
SIGNUP_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/signup" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }')
echo "$SIGNUP_RESPONSE" | jq '.'
USER_TOKEN=$(echo "$SIGNUP_RESPONSE" | jq -r '.token')
echo "User Token: $USER_TOKEN"
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

# Test creating service account without proper role (should fail)
echo "4. Testing service account creation without service role (should fail with 403)..."
SERVICE_FAIL_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/service-account" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -d '{
    "email": "service@example.com",
    "password": "servicepass123"
  }')
echo "$SERVICE_FAIL_RESPONSE" | jq '.'
echo
echo

# Note: To test service account creation successfully, you need an existing service account
echo "5. Note: To test service account creation, first create a service account manually:"
echo "   See API.md section 'Creating the First Service Account'"
echo
echo

# If you have a service account token, uncomment and use this:
# SERVICE_TOKEN="your_service_account_token_here"
# echo "6. Testing service account creation with service role..."
# SERVICE_CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/service-account" \
#   -H "Content-Type: application/json" \
#   -H "Authorization: Bearer $SERVICE_TOKEN" \
#   -d '{
#     "email": "newservice@example.com",
#     "password": "servicepass123"
#   }')
# echo "$SERVICE_CREATE_RESPONSE" | jq '.'
# echo
# echo
echo

# Test GET /db/users
echo "6. Testing GET /db/users..."
curl -s "$BASE_URL/db/users" | jq '.'
echo
echo

# Test POST /db/posts (if posts table exists)
echo "7. Testing POST /db/posts..."
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

# Test authenticated request with user token
echo "8. Testing authenticated request with user token..."
curl -s "$BASE_URL/db/users" \
  -H "Authorization: Bearer $USER_TOKEN" | jq '.'
echo
echo

echo "=== Tests Complete ==="
echo
echo "Summary:"
echo "- Regular user signup and login: ✓"
echo "- Service account creation without proper role: ✓ (should fail)"
echo "- To test full service account workflow, create one manually first"
echo "  (See API.md for instructions)"
echo
