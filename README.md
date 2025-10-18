# Project Kit

A full-stack Rust application framework with authentication, database ORM, and REST API.

## Features

- 🔐 **Authentication** - JWT-based auth with Argon2 password hashing
- 🗄️ **Custom ORM** - Type-safe database operations with SQLite and MySQL support
- 🔄 **Migrations** - Automatic database migrations on startup
- ⚙️ **Configuration** - TOML-based config with environment variable overrides
- 🚀 **REST API** - Built with Axum for high performance
- 📦 **Modular Architecture** - Clean separation of concerns
- 📁 **File Storage** - Transactional file upload/download with database tracking

## Quick Start

### 1. Configuration

Create or edit `projectkit.toml` in the project root:

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

### 2. Run the Server

```bash
cargo run --package server --bin server
```

The server will:
1. Load configuration from `projectkit.toml`
2. Connect to the database
3. Run pending migrations
4. Start the HTTP server

### 3. Test the API

```bash
./test_api.sh
```

Or manually:

```bash
# Signup
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}'

# Login
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}'
```

## Project Structure

```
projectkit/
├── crates/
│   ├── api/          # HTTP handlers and routing
│   ├── auth/         # Authentication service
│   ├── core/         # Configuration and shared utilities
│   ├── storage/      # File storage service with ORM integration
│   └── server/       # Main server binary and migrations
├── orm/              # Custom ORM library (workspace dependency)
├── projectkit.toml   # Configuration file
├── test_api.sh       # API test script
└── API.md           # Detailed API documentation
```

## Configuration

### File-based Configuration

Edit `projectkit.toml` to configure the application.

### Environment Variables

Override any configuration value using environment variables with the `PROJECTKIT_` prefix:

```bash
export PROJECTKIT_DATABASE_URL="postgres://user:pass@localhost:5432/projectkit"
export PROJECTKIT_AUTH_JWT_SECRET="my-production-secret"
export PROJECTKIT_SERVER_PORT=8080
export PROJECTKIT_STORAGE_PATH="./uploads"  # Optional, defaults to ./storage
```

Environment variables take precedence over the TOML file.

## Database Support

- **SQLite** - `sqlite:path/to/db.db` or `sqlite::memory:`
- **MySQL** - `mysql://user:pass@host:port/dbname`
- **PostgreSQL** - `postgres://user:pass@host:port/dbname` (coming soon)

## File Storage

The storage service provides transactional file management with database tracking:

- **Upload files** - Multipart form data upload with automatic metadata persistence
- **Download files** - Permission-checked file retrieval
- **Delete files** - Transactional deletion (removes both file and database record)
- **List files** - Query user's uploaded files
- **Storage stats** - Track total file count and storage usage per user

Files are stored on the local filesystem with metadata tracked in the database. The service uses a compensating transaction pattern to ensure consistency between filesystem and database operations.

## Migrations

Migrations are defined in `crates/server/src/migrations.rs` and run automatically on server startup.

To add a new migration:

1. Create a new struct implementing the `Migration` trait
2. Add it to the `run_migrations` function
3. Restart the server

Example:

```rust
struct CreateProductsTable;

#[async_trait]
impl Migration for CreateProductsTable {
    fn name(&self) -> &str {
        "create_products_table"
    }

    fn version(&self) -> i64 {
        20241018_000003
    }

    async fn up(&self, schema: &mut Schema) -> Result<()> {
        schema.create_table("products", |table| {
            table.id("id");
            table.string("name", 200);
            table.decimal("price", 10, 2);
            table.timestamps();
        });
        Ok(())
    }

    async fn down(&self, schema: &mut Schema) -> Result<()> {
        schema.drop_table("products");
        Ok(())
    }
}
```

## API Documentation

See [API.md](./API.md) for detailed API documentation.

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Examples

```bash
# ORM examples
cargo run --example blog_app
cargo run --example migrations_example
cargo run --example query_builder_demo
```

## License

MIT
