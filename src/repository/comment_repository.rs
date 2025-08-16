use anyhow::Result;
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use uuid::Uuid;
use crate::domain::comment::{Comment, EntityType};
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct CommentRepository {
    pool: Arc<SqlitePool>,
}

impl CommentRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, comment: &Comment) -> Result<()> {
        let id_str = comment.id.to_string();
        let entity_id_str = comment.entity_id.to_string();
        let entity_type_str = format!("{:?}", comment.entity_type);
        let author_id_str = comment.author_id.map(|id| id.to_string());
        let created_at_str = comment.created_at.to_rfc3339();
        let updated_at_str = comment.updated_at.to_rfc3339();
        let edited_int = comment.edited as i32;
        
        sqlx::query(
            r#"
            INSERT INTO comments (
                id, entity_id, entity_type, author_id, author_name, 
                content, created_at, updated_at, edited
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(id_str)
        .bind(entity_id_str)
        .bind(entity_type_str)
        .bind(author_id_str)
        .bind(&comment.author_name)
        .bind(&comment.content)
        .bind(created_at_str)
        .bind(updated_at_str)
        .bind(edited_int)
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn update(&self, comment: &Comment) -> Result<()> {
        let id_str = comment.id.to_string();
        let updated_at_str = comment.updated_at.to_rfc3339();
        let edited_int = comment.edited as i32;
        
        sqlx::query(
            r#"
            UPDATE comments 
            SET content = ?, updated_at = ?, edited = ?
            WHERE id = ?
            "#
        )
        .bind(&comment.content)
        .bind(updated_at_str)
        .bind(edited_int)
        .bind(id_str)
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Comment>> {
        let id_str = id.to_string();
        
        let row = sqlx::query(
            r#"
            SELECT id, entity_id, entity_type, author_id, author_name,
                   content, created_at, updated_at, edited
            FROM comments
            WHERE id = ?
            "#
        )
        .bind(id_str)
        .fetch_optional(&*self.pool)
        .await?;
        
        if let Some(row) = row {
            let comment = Comment {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                entity_id: Uuid::parse_str(&row.get::<String, _>("entity_id"))?,
                entity_type: if row.get::<String, _>("entity_type") == "Task" {
                    EntityType::Task
                } else {
                    EntityType::Goal
                },
                author_id: row.get::<Option<String>, _>("author_id")
                    .and_then(|id| Uuid::parse_str(&id).ok()),
                author_name: row.get("author_name"),
                content: row.get("content"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?.with_timezone(&Utc),
                edited: row.get::<i32, _>("edited") != 0,
                attachments: Vec::new(), // TODO: Store attachments separately if needed
            };
            Ok(Some(comment))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let id_str = id.to_string();
        
        let result = sqlx::query(
            "DELETE FROM comments WHERE id = ?"
        )
        .bind(id_str)
        .execute(&*self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }

    pub async fn list_for_entity(&self, entity_id: Uuid) -> Result<Vec<Comment>> {
        let entity_id_str = entity_id.to_string();
        
        let rows = sqlx::query(
            r#"
            SELECT id, entity_id, entity_type, author_id, author_name,
                   content, created_at, updated_at, edited
            FROM comments
            WHERE entity_id = ?
            ORDER BY created_at ASC
            "#
        )
        .bind(entity_id_str)
        .fetch_all(&*self.pool)
        .await?;
        
        let mut comments = Vec::new();
        for row in rows {
            let comment = Comment {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                entity_id: Uuid::parse_str(&row.get::<String, _>("entity_id"))?,
                entity_type: if row.get::<String, _>("entity_type") == "Task" {
                    EntityType::Task
                } else {
                    EntityType::Goal
                },
                author_id: row.get::<Option<String>, _>("author_id")
                    .and_then(|id| Uuid::parse_str(&id).ok()),
                author_name: row.get("author_name"),
                content: row.get("content"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?.with_timezone(&Utc),
                edited: row.get::<i32, _>("edited") != 0,
                attachments: Vec::new(),
            };
            comments.push(comment);
        }
        
        Ok(comments)
    }
}