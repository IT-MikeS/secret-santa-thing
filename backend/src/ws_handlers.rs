use axum::{
    extract::ws::{Message, WebSocket},
    extract::{Query, State, WebSocketUpgrade},
    response::Response,
    Json,
};
use futures::{sink::SinkExt, stream::StreamExt};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    db::DbPool,
    errors::{AppError, Result},
    models::{JoinGroupInput, WsMessage},
};

pub type Connections =
    Arc<RwLock<HashMap<String, HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>>;

pub async fn generate_pairs(
    State((pool, connections)): State<(DbPool, Connections)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<()> {
    let group_id = params
        .get("id")
        .ok_or_else(|| crate::errors::AppError::InvalidInput("Group ID required".to_string()))?;

    let mut tx = pool.begin().await?;

    #[derive(Clone)]
    struct Member {
        user_id: String,
        name: String,
    }

    let members = sqlx::query_as!(
        Member,
        r#"
    SELECT 
        user_id as "user_id!", 
        name as "name!"
    FROM members 
    WHERE group_id = ?
    "#,
        group_id
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut pairs = HashMap::new();
    let mut by_user_id = HashMap::new();

    let mut shuffled_members = members.clone();
    shuffled_members.shuffle(&mut rand::thread_rng());

    for i in 0..shuffled_members.len() {
        let giver = &shuffled_members[i];
        let receiver = &shuffled_members[(i + 1) % shuffled_members.len()];

        sqlx::query!(
            r#"
        INSERT INTO assignments (group_id, giver_id, receiver_id)
        VALUES (?, ?, ?)
        "#,
            group_id,
            giver.user_id,
            receiver.user_id
        )
        .execute(&mut *tx)
        .await?;

        pairs.insert(giver.user_id.clone(), receiver.user_id.clone());
        by_user_id.insert(giver.user_id.clone(), receiver.name.clone());
    }

    sqlx::query!(
        "UPDATE groups SET is_generated = TRUE WHERE id = ?",
        group_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    broadcast_to_group(
        &connections,
        group_id,
        WsMessage::AssignmentsGenerated { by_user_id, pairs },
    )
    .await?;

    Ok(())
}

pub async fn join_group(
    State((pool, connections)): State<(DbPool, Connections)>,
    Json(input): Json<JoinGroupInput>,
) -> Result<()> {
    let name = input.name.clone();
    let group_id = input.group_id.clone();
    let user_id = input.user_id.clone();

    // crate::db::join_group(&pool, &input.group_id, &input.user_id, &input.name).await?;
    let is_generated = sqlx::query_scalar!(
        "SELECT is_generated as \"is_generated!: bool\" FROM groups WHERE id = ?",
        group_id
    )
    .fetch_optional(&pool)
    .await?
    .unwrap_or(false);

    if is_generated {
        return Err(AppError::GroupAlreadyGenerated);
    }

    let name_exists = sqlx::query_scalar!(
        "SELECT (EXISTS(SELECT 1 FROM members WHERE group_id = ? AND name = ?))",
        group_id,
        name
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(0)
        > 0;

    if name_exists {
        return Err(AppError::NameTaken);
    }

    sqlx::query!(
        "INSERT INTO members (group_id, user_id, name, is_creator) VALUES (?, ?, ?, FALSE)",
        group_id,
        user_id,
        name
    )
    .execute(&pool)
    .await?;

    let group = crate::db::get_group_data(&pool, &group_id).await?;
    broadcast_to_group(&connections, &group_id, WsMessage::GroupUpdate(group)).await?;

    Ok(())
}

pub async fn message_handler(
    ws: WebSocketUpgrade,
    State((pool, connections)): State<(DbPool, Connections)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response> {
    let group_id = params
        .get("id")
        .ok_or_else(|| AppError::InvalidInput("Group ID required".to_string()))?;
    let user_id = params
        .get("userId")
        .ok_or_else(|| AppError::InvalidInput("User ID required".to_string()))?;

    let group_id = group_id.clone();
    let user_id = user_id.clone();

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, pool, connections, group_id, user_id)))
}

async fn handle_socket(
    socket: WebSocket,
    pool: DbPool,
    connections: Connections,
    group_id: String,
    user_id: String,
) {
    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    {
        let mut conns = connections.write().await;
        conns
            .entry(group_id.clone())
            .or_insert_with(HashMap::new)
            .insert(user_id.clone(), tx);
    }

    if let Ok(group) = crate::db::get_group_data(&pool, &group_id).await {
        let msg = serde_json::to_string(&WsMessage::GroupUpdate(group)).unwrap();
        let _ = sender.send(Message::Text(msg)).await;
    }

    let assignment = sqlx::query!(
        r#"
        SELECT m.name 
        FROM assignments a
        JOIN members m ON m.user_id = a.receiver_id
        WHERE a.group_id = ? AND a.giver_id = ?
        "#,
        group_id,
        user_id
    )
    .fetch_optional(&pool)
    .await;

    if let Ok(Some(row)) = assignment {
        let msg = serde_json::to_string(&WsMessage::assignment {
            receiver: row.name.unwrap_or_default(),
        })
        .unwrap();
        let _ = sender.send(Message::Text(msg)).await;
    }

    let mut recv_task =
        tokio::spawn(async move { while let Some(Ok(_)) = receiver.next().await {} });

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };

    let mut conns = connections.write().await;
    if let Some(group_conns) = conns.get_mut(&group_id) {
        group_conns.remove(&user_id);
        if group_conns.is_empty() {
            conns.remove(&group_id);
        }
    }
}

pub async fn broadcast_to_group(
    connections: &Connections,
    group_id: &str,
    message: WsMessage,
) -> Result<()> {
    let msg = serde_json::to_string(&message)?;
    let conns = connections.read().await;

    if let Some(group_conns) = conns.get(group_id) {
        for tx in group_conns.values() {
            let _ = tx.send(Message::Text(msg.clone()));
        }
    }

    Ok(())
}
