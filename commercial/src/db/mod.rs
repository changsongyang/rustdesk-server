use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::env;
use std::fs;

pub async fn init_database() -> Result<SqlitePool> {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "./data/rustdesk_pro.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let sql_content = fs::read_to_string("./scripts/init_db.sql")?;

    let statements: Vec<&str> = sql_content.split(';').collect();

    for stmt in statements {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await?;
        }
    }

    Ok(pool)
}
