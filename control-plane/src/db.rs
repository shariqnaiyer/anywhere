use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

pub async fn init(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS accounts (
            username        TEXT PRIMARY KEY NOT NULL,
            email           TEXT,
            cf_tunnel_id    TEXT NOT NULL,
            cf_tunnel_token TEXT NOT NULL,
            cf_dns_record_id TEXT NOT NULL,
            created_at      TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Account {
    pub username: String,
    pub email: Option<String>,
    pub cf_tunnel_id: String,
    pub cf_tunnel_token: String,
    pub cf_dns_record_id: String,
    pub created_at: String,
}

pub async fn username_taken(pool: &SqlitePool, username: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM accounts WHERE username = ? LIMIT 1")
            .bind(username)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

pub async fn insert_account(
    pool: &SqlitePool,
    username: &str,
    email: Option<&str>,
    cf_tunnel_id: &str,
    cf_tunnel_token: &str,
    cf_dns_record_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO accounts (username, email, cf_tunnel_id, cf_tunnel_token, cf_dns_record_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(username)
    .bind(email)
    .bind(cf_tunnel_id)
    .bind(cf_tunnel_token)
    .bind(cf_dns_record_id)
    .execute(pool)
    .await?;
    Ok(())
}
