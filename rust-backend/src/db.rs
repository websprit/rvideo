use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Clone, FromRow)]
pub struct DbUser {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_admin: i8,
    pub disable_premium: i8,
    pub created_at: DateTime<Utc>,
}

pub async fn init_database(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
          id INT AUTO_INCREMENT PRIMARY KEY,
          username VARCHAR(255) UNIQUE NOT NULL,
          password_hash VARCHAR(255) NOT NULL,
          is_admin TINYINT DEFAULT 0,
          disable_premium TINYINT DEFAULT 1,
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_data (
          user_id INT NOT NULL,
          data_key VARCHAR(100) NOT NULL,
          data_value LONGTEXT,
          updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
          PRIMARY KEY (user_id, data_key),
          FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    let existing: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
        .bind("admin")
        .fetch_optional(pool)
        .await?;

    if existing.is_none() {
        let hash = hash_password("Admin@1234")
            .await
            .map_err(sqlx::Error::Protocol)?;
        sqlx::query(
            "INSERT INTO users (username, password_hash, is_admin, disable_premium) VALUES (?, ?, 1, 0)",
        )
        .bind("admin")
        .bind(hash)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_user_by_username(
    pool: &MySqlPool,
    username: &str,
) -> Result<Option<DbUser>, sqlx::Error> {
    sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_by_id(pool: &MySqlPool, id: i64) -> Result<Option<DbUser>, sqlx::Error> {
    sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_all_users(pool: &MySqlPool) -> Result<Vec<DbUser>, sqlx::Error> {
    sqlx::query_as::<_, DbUser>(
        "SELECT id, username, password_hash, is_admin, disable_premium, created_at FROM users ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

pub async fn create_user(
    pool: &MySqlPool,
    username: &str,
    password: &str,
    disable_premium: bool,
) -> Result<DbUser, sqlx::Error> {
    let hash = hash_password(password)
        .await
        .map_err(sqlx::Error::Protocol)?;
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, disable_premium) VALUES (?, ?, ?)",
    )
    .bind(username)
    .bind(hash)
    .bind(if disable_premium { 1 } else { 0 })
    .execute(pool)
    .await?;

    get_user_by_id(pool, result.last_insert_id() as i64)
        .await?
        .ok_or_else(|| sqlx::Error::Protocol("created user not found".into()))
}

pub async fn update_user(
    pool: &MySqlPool,
    id: i64,
    username: Option<&str>,
    password: Option<&str>,
    disable_premium: Option<bool>,
) -> Result<bool, sqlx::Error> {
    let Some(current) = get_user_by_id(pool, id).await? else {
        return Ok(false);
    };

    let next_username = username.unwrap_or(&current.username).to_string();
    let next_password_hash = if let Some(password) = password {
        hash_password(password)
            .await
            .map_err(sqlx::Error::Protocol)?
    } else {
        current.password_hash
    };
    let next_disable_premium = disable_premium.unwrap_or(current.disable_premium == 1);

    let result = sqlx::query(
        "UPDATE users SET username = ?, password_hash = ?, disable_premium = ? WHERE id = ?",
    )
    .bind(next_username)
    .bind(next_password_hash)
    .bind(if next_disable_premium { 1 } else { 0 })
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_user(pool: &MySqlPool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = ? AND is_admin = 0")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_user_data(
    pool: &MySqlPool,
    user_id: i64,
    key: &str,
) -> Result<String, sqlx::Error> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT data_value FROM user_data WHERE user_id = ? AND data_key = ?")
            .bind(user_id)
            .bind(key)
            .fetch_optional(pool)
            .await?;

    Ok(row
        .and_then(|value| value.0)
        .unwrap_or_else(|| "{}".to_string()))
}

pub async fn set_user_data(
    pool: &MySqlPool,
    user_id: i64,
    key: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_data (user_id, data_key, data_value)
        VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE data_value = VALUES(data_value)
        "#,
    )
    .bind(user_id)
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn hash_password(password: &str) -> Result<String, String> {
    let password = password.to_string();
    tokio::task::spawn_blocking(move || {
        bcrypt::hash(password, 10).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let password = password.to_string();
    let hash = hash.to_string();
    tokio::task::spawn_blocking(move || {
        bcrypt::verify(password, &hash).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}
