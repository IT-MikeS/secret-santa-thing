use axum::{
    extract::{Query, State},
    Json,
};
use serde_json::json;
use std::collections::HashMap;

use crate::{db::DbPool, errors::Result, models::*};

pub async fn create_group(
    State(pool): State<DbPool>,
    Json(input): Json<CreateGroupInput>,
) -> Result<Json<serde_json::Value>> {
    let group_id = crate::db::create_group(&pool, input.name, input.creator, input.user_id).await?;
    Ok(Json(json!({"id": group_id})))
}

pub async fn get_group(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Group>> {
    let group_id = params
        .get("id")
        .ok_or_else(|| crate::errors::AppError::InvalidInput("Group ID required".to_string()))?;

    let group = crate::db::get_group_data(&pool, group_id).await?;
    Ok(Json(group))
}

pub async fn get_user_groups(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<UserGroupResponse>>> {
    let user_id = params
        .get("userId")
        .ok_or_else(|| crate::errors::AppError::InvalidInput("User ID required".to_string()))?;

    let groups = sqlx::query_as!(
        UserGroupResponse,
        r#"
      SELECT 
          g.id as "id!", 
          g.name as "name!", 
          g.is_generated as "is_generated!: bool", 
          m.is_creator as "is_creator!: bool"
      FROM groups g
      JOIN members m ON g.id = m.group_id
      WHERE m.user_id = ?
      ORDER BY g.id DESC
      "#,
        user_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(groups))
}
