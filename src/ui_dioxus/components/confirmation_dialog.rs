use dioxus::prelude::*;

#[component]
pub fn ConfirmationDialog(
    title: String,
    message: String,
    confirm_text: String,
    cancel_text: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
    danger: bool,
) -> Element {
    let confirm_color = if danger { "#dc2626" } else { "#3b82f6" };
    
    rsx! {
        // Modal backdrop
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
                   background: rgba(0, 0, 0, 0.5); z-index: 1000;
                   display: flex; align-items: center; justify-content: center;",
            onclick: move |_| on_cancel.call(()),
            
            // Modal content
            div {
                style: "background: white; border-radius: 12px; padding: 24px;
                       width: 90%; max-width: 400px;
                       box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);",
                onclick: move |e| e.stop_propagation(),
                
                // Title
                h3 {
                    style: "margin: 0 0 12px 0; font-size: 20px; font-weight: 600;",
                    "{title}"
                }
                
                // Message
                p {
                    style: "margin: 0 0 24px 0; color: #666; line-height: 1.5;",
                    "{message}"
                }
                
                // Buttons
                div {
                    style: "display: flex; justify-content: flex-end; gap: 10px;",
                    
                    button {
                        style: "padding: 8px 20px; border: 1px solid #ddd; 
                               background: white; color: #333; border-radius: 4px; 
                               cursor: pointer; font-size: 14px;",
                        onclick: move |_| on_cancel.call(()),
                        "{cancel_text}"
                    }
                    
                    button {
                        style: format!("padding: 8px 20px; border: none; 
                               background: {}; color: white; border-radius: 4px; 
                               cursor: pointer; font-size: 14px;", confirm_color),
                        onclick: move |_| on_confirm.call(()),
                        "{confirm_text}"
                    }
                }
            }
        }
    }
}