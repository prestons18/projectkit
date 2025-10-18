# Project Kit API Documentation

## Overview

Project Kit provides a REST API with authentication and dynamic database access using your custom ORM.

## Configuration

Project Kit uses a `projectkit.toml` configuration file in the project root:

```toml
[database]
url = "sqlite:projectkit.db"

[auth]
jwt_secret = "super-secret-key-change-in-production"
token_expiry_seconds = 3600

[server]
host = "0.0.0.0"
port = 3000
```

### Environment Variable Overrides

You can override any configuration value using environment variables with the `PROJECTKIT_` prefix:

```bash
export PROJECTKIT_DATABASE_URL="postgres://user:pass@localhost:5432/projectkit"
export PROJECTKIT_AUTH_JWT_SECRET="my-production-secret"
export PROJECTKIT_SERVER_PORT=8080
```

## Running the Server

```bash
cargo run --package server --bin server
```

The server will start on the configured host and port (default: `http://0.0.0.0:3000`)

## API Endpoints

### Authentication

#### POST /auth/signup
Register a new user with the default `user` role.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Response (201 Created):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id": 1,
    "email": "user@example.com"
  }
}
```

**Note:** The JWT token includes role information. Regular users get the `user` role by default.

#### POST /auth/login
Login an existing user.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Response (200 OK):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id": 1,
    "email": "user@example.com"
  }
}
```

#### POST /auth/service-account
Create a service account with the `service` role. **Requires authentication with an existing service account.**

**Request:**
```bash
curl -X POST http://localhost:3000/auth/service-account \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <SERVICE_ACCOUNT_TOKEN>" \
  -d '{
    "email": "service@example.com",
    "password": "securepassword"
  }'
```

**Response (201 Created):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id": 2,
    "email": "service@example.com"
  }
}
```

**Error Response (403 Forbidden):**
```json
{
  "error": "Access denied. Service role required to create service accounts."
}
```

**Note:** Only existing service accounts can create new service accounts. The first service account must be created directly in the database.

### Database Operations

#### GET /db/:table
Fetch all records from a table.

**Example:**
```bash
curl http://localhost:3000/db/users
```

**Response (200 OK):**
```json
[
  {
    "id": 1,
    "email": "user@example.com",
    "created_at": "2025-10-18T00:00:00Z"
  }
]
```

#### POST /db/:table
Insert a new record into a table.

**Example:**
```bash
curl -X POST http://localhost:3000/db/posts \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My First Post",
    "content": "Hello World",
    "user_id": 1
  }'
```

**Response (201 Created):**
```json
{
  "success": true,
  "rows_affected": 1
}
```

## Using the ORM Internally

The API uses your ORM internally. Example from the codebase:

```rust
use orm::prelude::*;

// In a handler
let backend = state.db.backend();

// Using ModelCrud trait
let users = User::all(backend).await?;

// Using query builder
let user = User::query(backend)
    .where_eq("email", QueryValue::String("user@example.com".to_string()))
    .first()
    .await?;
```

## Role-Based Access Control

Project Kit supports two user roles:

### Roles

- **`user`** - Regular users with basic permissions (default for signup)
- **`service`** - Service accounts for API-to-API communication with elevated privileges

### Using Roles in Requests

All authenticated requests include the user's role in the JWT token. The token is validated on each request.

**Example authenticated request:**
```bash
curl http://localhost:3000/db/users \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..."
```

### Protected Routes

Routes can be protected with role-based middleware:
- Some routes may require the `user` role
- Some routes may require the `service` role
- Some routes may accept any authenticated user

If you don't have the required role, you'll receive a `403 Forbidden` response:
```json
{
  "error": "Access denied. Required role: Service"
}
```

## File Storage

### POST /files/upload
Upload a file (requires authentication).

**Request:**
```bash
curl -X POST http://localhost:3000/files/upload \
  -H "Authorization: Bearer <TOKEN>" \
  -F "file=@/path/to/file.pdf"
```

**Response (201 Created):**
```json
{
  "success": true,
  "file": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "original_name": "file.pdf",
    "stored_name": "550e8400-e29b-41d4-a716-446655440000.pdf",
    "size": 102400,
    "mime_type": "application/pdf",
    "created_at": "2025-10-18T03:00:00Z"
  }
}
```

### GET /files/:id
Download a file (requires authentication and ownership).

**Request:**
```bash
curl http://localhost:3000/files/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <TOKEN>" \
  -o downloaded_file.pdf
```

**Response:**
- Binary file data with appropriate `Content-Type` and `Content-Disposition` headers

### DELETE /files/:id
Delete a file (requires authentication and ownership).

**Request:**
```bash
curl -X DELETE http://localhost:3000/files/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <TOKEN>"
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "File 550e8400-e29b-41d4-a716-446655440000 deleted successfully"
}
```

### GET /files
List all files for the authenticated user.

**Request:**
```bash
curl http://localhost:3000/files \
  -H "Authorization: Bearer <TOKEN>"
```

**Response (200 OK):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "original_name": "document.pdf",
    "stored_name": "550e8400-e29b-41d4-a716-446655440000.pdf",
    "size": 102400,
    "mime_type": "application/pdf",
    "created_at": "2025-10-18T03:00:00Z"
  }
]
```

### GET /files/stats
Get storage statistics for the authenticated user.

**Request:**
```bash
curl http://localhost:3000/files/stats \
  -H "Authorization: Bearer <TOKEN>"
```

**Response (200 OK):**
```json
{
  "file_count": 5,
  "total_size": 512000
}
```

## Database Setup

The server automatically runs migrations on startup, creating the necessary tables:
- `users` - For authentication (includes role column)
- `posts` - Example table with foreign key to users
- `files` - For file storage metadata with user ownership
- `migrations` - Tracks applied migrations

You can add custom migrations in `crates/server/src/migrations.rs`.

### Manual Table Creation (Optional)

If you prefer to create tables manually:

```sql
-- Users table (required for auth)
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Sessions table (optional, for session management)
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Files table (for file storage)
CREATE TABLE files (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    original_name TEXT NOT NULL,
    stored_name TEXT NOT NULL,
    size INTEGER NOT NULL,
    mime_type TEXT,
    storage_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

### Creating the First Service Account

Since only service accounts can create other service accounts, you need to create the first one manually:

```bash
# Using sqlite3 CLI
sqlite3 projectkit.db

# Insert a service account (you'll need to hash the password first)
INSERT INTO users (email, password_hash, role, created_at, updated_at)
VALUES (
  'admin@example.com',
  '<bcrypt_hash_of_password>',
  'service',
  datetime('now'),
  datetime('now')
);
```

Alternatively, temporarily modify the signup handler to create a service account, then revert the change.

## Error Responses

All errors return JSON with an `error` field:

```json
{
  "error": "Description of what went wrong"
}
```

Common status codes:
- `400 Bad Request`: Invalid input
- `401 Unauthorized`: Authentication failed or missing token
- `403 Forbidden`: Insufficient permissions (wrong role)
- `500 Internal Server Error`: Server or database error
