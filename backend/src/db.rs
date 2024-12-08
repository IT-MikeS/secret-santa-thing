use rand::Rng;
use sqlx::sqlite::SqlitePool;
use sqlx::{Pool, Sqlite};

use crate::errors::{AppError, Result};
use crate::models::{db_models::*, Group, Member};

pub type DbPool = Pool<Sqlite>;

pub async fn init_db() -> Result<DbPool> {
    let pool = SqlitePool::connect("sqlite:santa.db").await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT,
            creator TEXT,
            is_generated BOOLEAN DEFAULT FALSE
        );
        CREATE TABLE IF NOT EXISTS members (
            group_id TEXT,
            user_id TEXT,
            name TEXT,
            is_creator BOOLEAN,
            FOREIGN KEY(group_id) REFERENCES groups(id),
            UNIQUE(group_id, name),
            UNIQUE(group_id, user_id)
        );
        CREATE TABLE IF NOT EXISTS assignments (
            group_id TEXT,
            giver_id TEXT,
            receiver_id TEXT,
            FOREIGN KEY(group_id) REFERENCES groups(id),
            UNIQUE(group_id, giver_id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn get_group_data(pool: &DbPool, group_id: &str) -> Result<Group> {
    let group = sqlx::query_as!(
        DbGroup,
        r#"
        SELECT id as "id!", name as "name!", creator as "creator!", is_generated as "is_generated!"
        FROM groups 
        WHERE id = ?
        "#,
        group_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::GroupNotFound)?;

    let members = sqlx::query_as!(
        DbMember,
        r#"
        SELECT 
            group_id as "group_id!",
            user_id as "user_id!",
            name as "name!",
            is_creator as "is_creator!"
        FROM members 
        WHERE group_id = ?
        "#,
        group_id
    )
    .fetch_all(pool)
    .await?;

    Ok(Group {
        id: group.id,
        name: group.name,
        creator: group.creator,
        is_generated: group.is_generated,
        members: members
            .into_iter()
            .map(|m| Member {
                user_id: m.user_id,
                name: m.name,
                is_creator: m.is_creator,
            })
            .collect(),
    })
}

pub async fn create_group(
    pool: &DbPool,
    name: String,
    creator: String,
    user_id: String,
) -> Result<String> {
    let mut tx = pool.begin().await?;
    let group_id = generate_group_id();

    sqlx::query!(
        "INSERT INTO groups (id, name, creator) VALUES (?, ?, ?)",
        group_id,
        name,
        creator
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO members (group_id, user_id, name, is_creator) VALUES (?, ?, ?, TRUE)",
        group_id,
        user_id,
        creator
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(group_id)
}

fn generate_group_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();

    (0..5)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
