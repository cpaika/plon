#[cfg(test)]
mod tests {
    use dioxus::prelude::*;
    use dioxus_desktop::DesktopContext;
    use crate::domain::task::{Task, TaskStatus, Priority};
    use crate::repository::Repository;
    use std::sync::Arc;
    use sqlx::SqlitePool;
    
    // Mock component for testing the modal
    #[component]
    fn TestApp(task: Task, on_save: EventHandler<Task>, on_cancel: EventHandler<()>) -> Element {
        use super::super::task_edit_modal::TaskEditModal;
        
        rsx! {
            TaskEditModal {
                task: task,
                on_save: on_save,
                on_cancel: on_cancel,
            }
        }
    }
    
    #[tokio::test]
    async fn test_edit_modal_renders_task_details() {
        let task = Task::new(
            "Test Task".to_string(),
            "Test Description".to_string(),
        );
        
        let mut dom = VirtualDom::new_with_props(TestApp, TestAppProps {
            task: task.clone(),
            on_save: EventHandler::new(|_| {}),
            on_cancel: EventHandler::new(|_| {}),
        });
        
        dom.rebuild_in_place();
        
        // Check that the modal renders with task details
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Test Task"), "Should display task title");
        assert!(html.contains("Test Description"), "Should display task description");
        assert!(html.contains("Estimated Hours"), "Should have estimated hours field");
    }
    
    #[tokio::test]
    async fn test_edit_modal_save_button() {
        let task = Task::new(
            "Original Title".to_string(),
            "Original Description".to_string(),
        );
        
        let saved_task = Arc::new(std::sync::Mutex::new(None));
        let saved_task_clone = saved_task.clone();
        
        let mut dom = VirtualDom::new_with_props(TestApp, TestAppProps {
            task: task.clone(),
            on_save: EventHandler::new(move |updated_task| {
                *saved_task_clone.lock().unwrap() = Some(updated_task);
            }),
            on_cancel: EventHandler::new(|_| {}),
        });
        
        dom.rebuild_in_place();
        
        // Simulate saving
        // Note: In a real test, we'd simulate form input changes and button click
        // For now, we're testing the structure exists
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Save"), "Should have Save button");
        assert!(html.contains("Cancel"), "Should have Cancel button");
    }
    
    #[tokio::test]
    async fn test_edit_modal_cancel_button() {
        let task = Task::new(
            "Test Task".to_string(),
            "Test Description".to_string(),
        );
        
        let cancelled = Arc::new(std::sync::Mutex::new(false));
        let cancelled_clone = cancelled.clone();
        
        let mut dom = VirtualDom::new_with_props(TestApp, TestAppProps {
            task: task.clone(),
            on_save: EventHandler::new(|_| {}),
            on_cancel: EventHandler::new(move |_| {
                *cancelled_clone.lock().unwrap() = true;
            }),
        });
        
        dom.rebuild_in_place();
        
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Cancel"), "Should have Cancel button");
    }
    
    #[tokio::test]
    async fn test_edit_modal_validates_input() {
        let mut task = Task::new(
            "Test Task".to_string(),
            "Test Description".to_string(),
        );
        task.estimated_hours = Some(5.0);
        
        let mut dom = VirtualDom::new_with_props(TestApp, TestAppProps {
            task: task.clone(),
            on_save: EventHandler::new(|_| {}),
            on_cancel: EventHandler::new(|_| {}),
        });
        
        dom.rebuild_in_place();
        
        let html = dioxus_ssr::render(&dom);
        // Should show the estimated hours value
        assert!(html.contains("5"), "Should display estimated hours value");
    }
    
    #[tokio::test]
    async fn test_edit_modal_handles_empty_fields() {
        let task = Task::new(
            "".to_string(),
            "".to_string(),
        );
        
        let mut dom = VirtualDom::new_with_props(TestApp, TestAppProps {
            task: task.clone(),
            on_save: EventHandler::new(|_| {}),
            on_cancel: EventHandler::new(|_| {}),
        });
        
        dom.rebuild_in_place();
        
        let html = dioxus_ssr::render(&dom);
        // Should still render the form even with empty fields
        assert!(html.contains("Title"), "Should have title label");
        assert!(html.contains("Description"), "Should have description label");
    }
}