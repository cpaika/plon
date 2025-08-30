use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comment {
    pub id: Uuid,
    pub entity_id: Uuid, // Can be task_id or goal_id
    pub entity_type: EntityType,
    pub author_id: Option<Uuid>,
    pub author_name: String,
    pub content: String, // Markdown
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub edited: bool,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Task,
    Goal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attachment {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: usize,
    pub url: String,
}

impl Comment {
    pub fn new(
        entity_id: Uuid,
        entity_type: EntityType,
        author_name: String,
        content: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entity_id,
            entity_type,
            author_id: None,
            author_name,
            content,
            created_at: now,
            updated_at: now,
            edited: false,
            attachments: Vec::new(),
        }
    }

    pub fn edit(&mut self, new_content: String) {
        self.content = new_content;
        self.edited = true;
        self.updated_at = Utc::now();
    }

    pub fn add_attachment(
        &mut self,
        filename: String,
        mime_type: String,
        size_bytes: usize,
        url: String,
    ) {
        self.attachments.push(Attachment {
            id: Uuid::new_v4(),
            filename,
            mime_type,
            size_bytes,
            url,
        });
        self.updated_at = Utc::now();
    }

    pub fn remove_attachment(&mut self, attachment_id: Uuid) -> bool {
        let original_len = self.attachments.len();
        self.attachments.retain(|a| a.id != attachment_id);
        let removed = self.attachments.len() < original_len;
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_comment() {
        let task_id = Uuid::new_v4();
        let comment = Comment::new(
            task_id,
            EntityType::Task,
            "Alice".to_string(),
            "This looks good!".to_string(),
        );

        assert_eq!(comment.entity_id, task_id);
        assert_eq!(comment.entity_type, EntityType::Task);
        assert_eq!(comment.author_name, "Alice");
        assert_eq!(comment.content, "This looks good!");
        assert!(!comment.edited);
        assert!(comment.attachments.is_empty());
    }

    #[test]
    fn test_edit_comment() {
        let mut comment = Comment::new(
            Uuid::new_v4(),
            EntityType::Goal,
            "Bob".to_string(),
            "Initial comment".to_string(),
        );

        assert!(!comment.edited);
        let original_updated = comment.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        comment.edit("Updated comment".to_string());

        assert_eq!(comment.content, "Updated comment");
        assert!(comment.edited);
        assert!(comment.updated_at > original_updated);
    }

    #[test]
    fn test_attachments() {
        let mut comment = Comment::new(
            Uuid::new_v4(),
            EntityType::Task,
            "Charlie".to_string(),
            "See attachment".to_string(),
        );

        comment.add_attachment(
            "document.pdf".to_string(),
            "application/pdf".to_string(),
            1024000,
            "/uploads/document.pdf".to_string(),
        );

        assert_eq!(comment.attachments.len(), 1);
        assert_eq!(comment.attachments[0].filename, "document.pdf");
        assert_eq!(comment.attachments[0].size_bytes, 1024000);

        let attachment_id = comment.attachments[0].id;
        assert!(comment.remove_attachment(attachment_id));
        assert!(comment.attachments.is_empty());

        assert!(!comment.remove_attachment(Uuid::new_v4()));
    }
}
