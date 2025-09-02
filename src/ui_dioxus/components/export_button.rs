use dioxus::prelude::*;
use crate::services::{ExportService, ExportFormat};
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use std::sync::Arc;

#[component]
pub fn ExportButton() -> Element {
    let repository = use_context::<Arc<Repository>>();
    let mut show_menu = use_signal(|| false);
    let mut exporting = use_signal(|| false);
    let mut export_message = use_signal(|| None::<String>);
    
    let do_export = {
        let repo = repository.clone();
        let exporting = exporting.clone();
        let show_menu = show_menu.clone();
        let export_message = export_message.clone();
        
        move |format: ExportFormat| {
            let repo = repo.clone();
            let mut exporting = exporting.clone();
            let mut show_menu = show_menu.clone();
            let mut export_message = export_message.clone();
            
            spawn(async move {
                exporting.set(true);
                show_menu.set(false);
                
                let service = ExportService::new(repo);
                let filters = TaskFilters {
                    status: None,
                    assigned_resource_id: None,
                    goal_id: None,
                    overdue: false,
                    limit: None,
                };
                
                let (content_result, extension) = match format {
                    ExportFormat::Json => (service.export_to_json(filters).await, "json"),
                    ExportFormat::Csv => (service.export_to_csv(filters).await, "csv"),
                    ExportFormat::Markdown => (service.export_to_markdown(filters).await, "md"),
                };
                
                let result = match content_result {
                    Ok(content) => {
                        let filename = format!("tasks_export_{}.{}", 
                            chrono::Utc::now().format("%Y%m%d_%H%M%S"), extension);
                        match std::fs::write(&filename, content) {
                            Ok(_) => Ok(format!("✅ Exported to {}", filename)),
                            Err(e) => Err(format!("Failed to save file: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("Export failed: {}", e)),
                };
                
                match result {
                    Ok(msg) => {
                        export_message.set(Some(msg));
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        export_message.set(None);
                    }
                    Err(msg) => {
                        export_message.set(Some(msg));
                    }
                }
                
                exporting.set(false);
            });
        }
    };
    
    rsx! {
        div {
            style: "position: relative; display: inline-block;",
            
            // Export button
            button {
                style: "padding: 8px 16px; background: #6366f1; color: white; \
                       border: none; border-radius: 6px; cursor: pointer; \
                       font-size: 14px; font-weight: 500; display: flex; \
                       align-items: center; gap: 6px;",
                onclick: move |_| {
                    let current = *show_menu.read();
                    show_menu.set(!current);
                },
                disabled: *exporting.read(),
                
                if *exporting.read() {
                    "⏳ Exporting..."
                } else {
                    "📥 Export"
                }
            }
            
            // Dropdown menu
            if *show_menu.read() && !*exporting.read() {
                div {
                    style: "position: absolute; top: 100%; right: 0; margin-top: 4px; \
                           background: white; border: 1px solid #e5e7eb; border-radius: 6px; \
                           box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); z-index: 10; \
                           min-width: 150px;",
                    
                    button {
                        style: "display: block; width: 100%; padding: 8px 12px; \
                               text-align: left; background: none; border: none; \
                               cursor: pointer; font-size: 14px; \
                               hover: background: #f3f4f6;",
                        onclick: {
                            let export = do_export.clone();
                            move |_| export(ExportFormat::Json)
                        },
                        "📄 Export as JSON"
                    }
                    
                    button {
                        style: "display: block; width: 100%; padding: 8px 12px; \
                               text-align: left; background: none; border: none; \
                               cursor: pointer; font-size: 14px; \
                               hover: background: #f3f4f6;",
                        onclick: {
                            let export = do_export.clone();
                            move |_| export(ExportFormat::Csv)
                        },
                        "📊 Export as CSV"
                    }
                    
                    button {
                        style: "display: block; width: 100%; padding: 8px 12px; \
                               text-align: left; background: none; border: none; \
                               cursor: pointer; font-size: 14px; \
                               hover: background: #f3f4f6;",
                        onclick: {
                            let export = do_export.clone();
                            move |_| export(ExportFormat::Markdown)
                        },
                        "📝 Export as Markdown"
                    }
                }
            }
            
            // Export message
            if let Some(message) = export_message.read().as_ref() {
                div {
                    style: "position: absolute; top: 100%; right: 0; margin-top: 4px; \
                           padding: 8px 12px; background: white; border: 1px solid #e5e7eb; \
                           border-radius: 6px; box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1); \
                           white-space: nowrap; font-size: 14px;",
                    {message.clone()}
                }
            }
        }
    }
}