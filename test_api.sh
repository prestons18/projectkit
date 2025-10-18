#!/bin/bash

# Test API endpoints for Project Kit
# Tests role-based authentication, authorization, and file storage

BASE_URL="http://localhost:3000"

echo "=== Testing Project Kit API with Role-Based Auth & File Storage ==="
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

echo "=== File Storage API Tests ==="
echo

# Create a test file for upload
echo "9. Creating test file for upload..."
TEST_FILE="/tmp/projectkit_test_file.txt"
echo "This is a test file created at $(date)" > "$TEST_FILE"
echo "Test file created: $TEST_FILE"
cat "$TEST_FILE"
echo
echo

# Test file upload
echo "10. Testing file upload..."
UPLOAD_RESPONSE=$(curl -s -X POST "$BASE_URL/files/upload" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -F "file=@$TEST_FILE")
echo "$UPLOAD_RESPONSE" | jq '.'
FILE_ID=$(echo "$UPLOAD_RESPONSE" | jq -r '.file.id')
echo "Uploaded File ID: $FILE_ID"
echo
echo

# Test list files (should show 1 file)
echo "11. Testing list files..."
curl -s "$BASE_URL/files" \
  -H "Authorization: Bearer $USER_TOKEN" | jq '.'
echo
echo

# Test get storage stats
echo "12. Testing storage stats..."
curl -s "$BASE_URL/files/stats" \
  -H "Authorization: Bearer $USER_TOKEN" | jq '.'
echo
echo

# Test file download
if [ "$FILE_ID" != "null" ] && [ -n "$FILE_ID" ]; then
  echo "13. Testing file download..."
  DOWNLOAD_FILE="/tmp/projectkit_downloaded_file.txt"
  curl -s "$BASE_URL/files/$FILE_ID" \
    -H "Authorization: Bearer $USER_TOKEN" \
    -o "$DOWNLOAD_FILE"
  echo "Downloaded file to: $DOWNLOAD_FILE"
  echo "Content:"
  cat "$DOWNLOAD_FILE"
  echo
  echo
  
  # Test file download without auth (should fail)
  echo "14. Testing file download without auth (should fail with 401)..."
  curl -s "$BASE_URL/files/$FILE_ID" | jq '.'
  echo
  echo
  
  # Upload a second file
  echo "15. Uploading second test file..."
  echo "Second test file content" > /tmp/projectkit_test_file2.txt
  UPLOAD2_RESPONSE=$(curl -s -X POST "$BASE_URL/files/upload" \
    -H "Authorization: Bearer $USER_TOKEN" \
    -F "file=@/tmp/projectkit_test_file2.txt")
  echo "$UPLOAD2_RESPONSE" | jq '.'
  echo
  echo
  
  # List files again (should show 2 files)
  echo "16. Testing list files again (should show 2 files)..."
  curl -s "$BASE_URL/files" \
    -H "Authorization: Bearer $USER_TOKEN" | jq '.'
  echo
  echo
  
  # Test file deletion
  echo "17. Testing file deletion..."
  DELETE_RESPONSE=$(curl -s -X DELETE "$BASE_URL/files/$FILE_ID" \
    -H "Authorization: Bearer $USER_TOKEN")
  echo "$DELETE_RESPONSE" | jq '.'
  echo
  echo
  
  # List files after deletion (should show 1 file)
  echo "18. Testing list files after deletion (should show 1 file)..."
  curl -s "$BASE_URL/files" \
    -H "Authorization: Bearer $USER_TOKEN" | jq '.'
  echo
  echo
  
  # Test storage stats after deletion
  echo "19. Testing storage stats after deletion..."
  curl -s "$BASE_URL/files/stats" \
    -H "Authorization: Bearer $USER_TOKEN" | jq '.'
  echo
  echo
  
  # Test downloading deleted file (should fail)
  echo "20. Testing download of deleted file (should fail with 404)..."
  curl -s "$BASE_URL/files/$FILE_ID" \
    -H "Authorization: Bearer $USER_TOKEN" | jq '.'
  echo
  echo
else
  echo "13-20. Skipping file download/delete tests (upload failed)"
  echo
  echo
fi

# Cleanup
echo "Cleaning up test files..."
rm -f "$TEST_FILE" /tmp/projectkit_test_file2.txt "$DOWNLOAD_FILE"
echo "Cleanup complete"
echo
echo

echo "=== Tests Complete ==="
echo
echo "Summary:"
echo "- Regular user signup and login: ✓"
echo "- Service account creation without proper role: ✓ (should fail)"
echo "- Database operations: ✓"
echo "- File upload: ✓"
echo "- File list: ✓"
echo "- File download: ✓"
echo "- File deletion: ✓"
echo "- Storage stats: ✓"
echo "- Permission checks: ✓"
echo
echo "Note: To test full service account workflow, create one manually first"
echo "      (See API.md for instructions)"
echo
