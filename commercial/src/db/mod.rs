use anyhow::Result;
use bcrypt::{hash, DEFAULT_COST};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::env;
use std::fs;
use std::path::Path;

pub async fn init_database() -> Result<SqlitePool> {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "./data/rustdesk_pro.db".to_string());

    let path = Path::new(&database_url);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let connection_string = format!("sqlite:{}", database_url);
    
    println!("DEBUG: Database URL: {}", database_url);
    println!("DEBUG: Connection string: {}", connection_string);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    let sql_content = fs::read_to_string("./scripts/init_db.sql")?;

    let statements: Vec<&str> = sql_content.split(';').collect();

    for stmt in statements {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await?;
        }
    }

    init_admin_user_if_not_exists(&pool).await?;

    Ok(pool)
}

async fn init_admin_user_if_not_exists(pool: &SqlitePool) -> Result<()> {
    let admin_username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    let admin_email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@rustdesk.local".to_string());

    let exists: Option<(String,)> = sqlx::query_as("SELECT username FROM users WHERE username = ?")
        .bind(&admin_username)
        .fetch_optional(pool)
        .await?;

    if exists.is_none() {
        let password_hash = hash(&admin_password, DEFAULT_COST)?;
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, created_at, is_active)
            VALUES (?, ?, ?, ?, 'admin', datetime('now'), 1)
            "#,
        )
        .bind(&id)
        .bind(&admin_username)
        .bind(&admin_email)
        .bind(&password_hash)
        .execute(pool)
        .await?;

        log::info!("Admin user '{}' created successfully", admin_username);
    }

    Ok(())
}
