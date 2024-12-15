use crate::error::ApiError;
use crate::models::{Conversation, ConversationDetails, Message, NewConversation, NewMessage};
use axum::extract::{Json, Path, State};
use sqlx::SqlitePool;

pub async fn create_conversation(
    State(pool): State<SqlitePool>,
    Json(new_conv): Json<NewConversation>,
) -> Result<Json<Conversation>, ApiError> {
    let mut tx = pool.begin().await.map_err(ApiError::DatabaseError)?;

    let conversation = sqlx::query_as!(
        Conversation,
        r#"
        INSERT INTO conversations (property_id)
        VALUES (?)
        RETURNING id, property_id, created_at
        "#,
        new_conv.property_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::DatabaseError)?;

    // Add participants
    for user_id in new_conv.participant_ids {
        sqlx::query!(
            r#"
            INSERT INTO conversation_participants (conversation_id, user_id)
            VALUES (?, ?)
            "#,
            conversation.id,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(ApiError::DatabaseError)?;
    }

    tx.commit().await.map_err(ApiError::DatabaseError)?;

    Ok(Json(conversation))
}

pub async fn send_message(
    State(pool): State<SqlitePool>,
    Path(conv_id): Path<i64>,
    Json(new_message): Json<NewMessage>,
) -> Result<Json<Message>, ApiError> {
    let message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (conversation_id, sender_id, content, read)
        VALUES (?, ?, ?, false)
        RETURNING id, conversation_id, sender_id, content, read, created_at
        "#,
        conv_id,
        new_message.sender_id,
        new_message.content
    )
    .fetch_one(&pool)
    .await
    .map_err(ApiError::DatabaseError)?;

    Ok(Json(message))
}

pub async fn get_messages(
    State(pool): State<SqlitePool>,
    Path(conv_id): Path<i64>,
) -> Result<Json<Vec<Message>>, ApiError> {
    let messages = sqlx::query_as!(
        Message,
        r#"
        SELECT id, conversation_id, sender_id, content, read, created_at
        FROM messages
        WHERE conversation_id = ?
        ORDER BY created_at ASC
        "#,
        conv_id
    )
    .fetch_all(&pool)
    .await
    .map_err(ApiError::DatabaseError)?;

    Ok(Json(messages))
}

pub async fn get_user_conversations(
    State(pool): State<SqlitePool>,
    Path(user_id): Path<i64>,
) -> Result<Json<Vec<ConversationDetails>>, ApiError> {
    let conversations = sqlx::query_as!(
        ConversationDetails,
        r#"
        SELECT 
            c.id, 
            c.property_id,
            c.created_at,
            p.title as property_title,
            (
                SELECT COUNT(*)
                FROM messages m
                WHERE m.conversation_id = c.id
                AND m.read = false
                AND m.sender_id != ?
            ) as unread_count
        FROM conversations c
        JOIN conversation_participants cp ON c.id = cp.conversation_id
        JOIN properties p ON c.property_id = p.id
        WHERE cp.user_id = ?
        "#,
        user_id,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(ApiError::DatabaseError)?;

    Ok(Json(conversations))
}
