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
Register a new user.

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

## Database Setup

The server automatically runs migrations on startup, creating the necessary tables:
- `users` - For authentication
- `posts` - Example table with foreign key to users
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
```

## Error Responses

All errors return JSON with an `error` field:

```json
{
  "error": "Description of what went wrong"
}
```

Common status codes:
- `400 Bad Request`: Invalid input
- `401 Unauthorized`: Authentication failed
- `500 Internal Server Error`: Server or database error
