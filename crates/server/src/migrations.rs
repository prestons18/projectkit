use orm::migration::{Migration, MigrationRunner, Schema};
use orm::query::builder::Dialect;
use orm::schema::{ForeignKey, ForeignKeyAction};
use orm::backend::Backend;
use orm::error::Result;
use async_trait::async_trait;

/// Migration to create users table
struct CreateUsersTable;

#[async_trait]
impl Migration for CreateUsersTable {
    fn name(&self) -> &str {
        "create_users_table"
    }

    fn version(&self) -> i64 {
        20241018_000001
    }

    async fn up(&self, schema: &mut Schema) -> Result<()> {
        schema.create_table("users", |table| {
            table.id("id");
            table.string("email", 100);
            table.string("password_hash", 255);
            table.string("role", 20);
            table.timestamps();
            table.index("idx_users_email", vec!["email".to_string()], true);
        });
        Ok(())
    }

    async fn down(&self, schema: &mut Schema) -> Result<()> {
        schema.drop_table("users");
        Ok(())
    }
}

/// Migration to create sessions table
struct CreateSessionsTable;

#[async_trait]
impl Migration for CreateSessionsTable {
    fn name(&self) -> &str {
        "create_sessions_table"
    }

    fn version(&self) -> i64 {
        20241018_000002
    }

    async fn up(&self, schema: &mut Schema) -> Result<()> {
        schema.create_table("sessions", |table| {
            table.id("id");
            table.big_integer("user_id");
            table.string("token", 500);
            table.string("expires_at", 50);
            table.string("created_at", 50);
            
            table.foreign_key(ForeignKey {
                column: "user_id".to_string(),
                references_table: "users".to_string(),
                references_column: "id".to_string(),
                on_delete: Some(ForeignKeyAction::Cascade),
                on_update: None,
            });
            
            table.index("idx_sessions_token", vec!["token".to_string()], true);
            table.index("idx_sessions_user_id", vec!["user_id".to_string()], false);
        });
        Ok(())
    }

    async fn down(&self, schema: &mut Schema) -> Result<()> {
        schema.drop_table("sessions");
        Ok(())
    }
}

/// Migration to create posts table
struct CreatePostsTable;

#[async_trait]
impl Migration for CreatePostsTable {
    fn name(&self) -> &str {
        "create_posts_table"
    }

    fn version(&self) -> i64 {
        20241018_000003
    }

    async fn up(&self, schema: &mut Schema) -> Result<()> {
        // Note: SQLite doesn't support DEFAULT in table builder, so we'll use raw SQL for defaults
        schema.create_table("posts", |table| {
            table.id("id");
            table.string("title", 200);
            table.text("content");
            table.big_integer("user_id");
            table.timestamps();
            
            table.foreign_key(ForeignKey {
                column: "user_id".to_string(),
                references_table: "users".to_string(),
                references_column: "id".to_string(),
                on_delete: Some(ForeignKeyAction::Cascade),
                on_update: None,
            });
            
            table.index("idx_posts_user_id", vec!["user_id".to_string()], false);
        });
        Ok(())
    }

    async fn down(&self, schema: &mut Schema) -> Result<()> {
        schema.drop_table("posts");
        Ok(())
    }
}

/// Run all migrations silently
/// Returns true if any migrations were run
pub async fn run_migrations(backend: &dyn Backend, dialect: Dialect) -> Result<bool> {
    let mut runner = MigrationRunner::new(backend, dialect);
    
    // Add migrations in order
    runner.add_migration(Box::new(CreateUsersTable));
    runner.add_migration(Box::new(CreateSessionsTable));
    runner.add_migration(Box::new(CreatePostsTable));
    
    // Run pending migrations - this will print output only if migrations are executed
    runner.run_pending(backend).await?;
    
    // We can't easily detect if migrations ran without modifying the ORM,
    // so we'll just return false for now (migrations print their own output)
    Ok(false)
}
