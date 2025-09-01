use dioxus::prelude::*;
use crate::domain::claude_code::ClaudeCodeConfig;
use crate::repository::Repository;
use std::sync::Arc;

#[component]
pub fn ClaudeConfigAdmin() -> Element {
    // Get repository from context
    let repository = use_context::<Arc<Repository>>();
    let mut loading = use_signal(|| true);
    let mut config = use_signal(|| None::<ClaudeCodeConfig>);
    let mut save_status = use_signal(|| String::new());
    let mut show_api_key = use_signal(|| false);
    let mut show_github_token = use_signal(|| false);

    // Form fields
    let mut github_repo = use_signal(String::new);
    let mut github_owner = use_signal(String::new);
    let mut github_token = use_signal(String::new);
    let mut claude_api_key = use_signal(String::new);
    let mut default_base_branch = use_signal(String::new);
    let mut auto_create_pr = use_signal(|| true);
    let mut working_directory = use_signal(String::new);
    let mut claude_model = use_signal(String::new);
    let mut max_session_duration = use_signal(|| 60);

    // Load current config on mount
    use_effect({
        let repo = repository.clone();
        let mut loading_signal = loading.clone();
        let mut config_signal = config.clone();
        let mut github_repo_signal = github_repo.clone();
        let mut github_owner_signal = github_owner.clone();
        let mut github_token_signal = github_token.clone();
        let mut claude_api_key_signal = claude_api_key.clone();
        let mut default_base_branch_signal = default_base_branch.clone();
        let mut auto_create_pr_signal = auto_create_pr.clone();
        let mut working_directory_signal = working_directory.clone();
        let mut claude_model_signal = claude_model.clone();
        let mut max_session_duration_signal = max_session_duration.clone();
        let mut save_status_signal = save_status.clone();
        move || {
            let repo = repo.clone();
            spawn(async move {
            match repo.claude_code.get_config().await {
                Ok(Some(cfg)) => {
                    github_repo_signal.set(cfg.github_repo.clone());
                    github_owner_signal.set(cfg.github_owner.clone());
                    github_token_signal.set(cfg.github_token.clone().unwrap_or_default());
                    claude_api_key_signal.set(cfg.claude_api_key.clone().unwrap_or_default());
                    default_base_branch_signal.set(cfg.default_base_branch.clone());
                    auto_create_pr_signal.set(cfg.auto_create_pr);
                    working_directory_signal.set(cfg.working_directory.clone().unwrap_or_default());
                    claude_model_signal.set(cfg.claude_model.clone());
                    max_session_duration_signal.set(cfg.max_session_duration_minutes);
                    config_signal.set(Some(cfg));
                    loading_signal.set(false);
                }
                Ok(None) => {
                    // No config exists, create default
                    let new_config = ClaudeCodeConfig::new(
                        "your-repo".to_string(),
                        "your-username".to_string(),
                    );
                    github_repo_signal.set(new_config.github_repo.clone());
                    github_owner_signal.set(new_config.github_owner.clone());
                    default_base_branch_signal.set(new_config.default_base_branch.clone());
                    auto_create_pr_signal.set(new_config.auto_create_pr);
                    claude_model_signal.set(new_config.claude_model.clone());
                    max_session_duration_signal.set(new_config.max_session_duration_minutes);
                    config_signal.set(Some(new_config));
                    loading_signal.set(false);
                }
                Err(e) => {
                    save_status_signal.set(format!("Error loading config: {}", e));
                    loading_signal.set(false);
                }
            }
            });
        }
    });


    if loading() {
        return rsx! {
            div { class: "flex justify-center items-center h-screen",
                div { class: "text-xl", "Loading configuration..." }
            }
        };
    }

    rsx! {
        div { class: "max-w-4xl mx-auto p-6",
            h1 { class: "text-3xl font-bold mb-6", "Claude Code Configuration" }
            
            div { class: "bg-white rounded-lg shadow-md p-6",
                // GitHub Settings
                div { class: "mb-6",
                    h2 { class: "text-xl font-semibold mb-4 text-gray-700", "GitHub Settings" }
                    
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Repository Owner" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{github_owner}",
                                oninput: move |e| github_owner.set(e.value())
                            }
                        }
                        
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Repository Name" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{github_repo}",
                                oninput: move |e| github_repo.set(e.value())
                            }
                        }
                        
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Default Base Branch" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{default_base_branch}",
                                placeholder: "main",
                                oninput: move |e| default_base_branch.set(e.value())
                            }
                        }
                        
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", 
                                "GitHub Token ",
                                span { class: "text-xs text-gray-500", "(Optional - for private repos)" }
                            }
                            div { class: "relative",
                                input {
                                    r#type: if show_github_token() { "text" } else { "password" },
                                    class: "w-full px-3 py-2 pr-10 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    value: "{github_token}",
                                    placeholder: "ghp_...",
                                    oninput: move |e| github_token.set(e.value())
                                }
                                button {
                                    r#type: "button",
                                    class: "absolute right-2 top-2 text-gray-500 hover:text-gray-700",
                                    onclick: move |_| show_github_token.set(!show_github_token()),
                                    if show_github_token() { "👁" } else { "🔒" }
                                }
                            }
                        }
                    }
                }

                // Workspace Settings
                div { class: "mb-6",
                    h2 { class: "text-xl font-semibold mb-4 text-gray-700", "Workspace Settings" }
                    
                    div { class: "space-y-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", 
                                "Workspace Root Directory ",
                                span { class: "text-xs text-gray-500", "(Default: ~/plon-workspaces)" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{working_directory}",
                                placeholder: "~/plon-workspaces",
                                oninput: move |e| working_directory.set(e.value())
                            }
                            p { class: "text-xs text-gray-500 mt-1",
                                "Each task will be cloned into a subfolder like: task-[id]-[title]"
                            }
                        }
                        
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "Custom Git Clone URL ",
                                span { class: "text-xs text-gray-500", "(Optional - overrides GitHub settings)" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: {format!("https://github.com/{}/{}.git", github_owner(), github_repo())},
                                placeholder: "https://github.com/owner/repo.git",
                                disabled: true
                            }
                            if !github_owner().is_empty() && !github_repo().is_empty() {
                                p { class: "text-xs text-gray-500 mt-1",
                                    {format!("Will use: https://github.com/{}/{}.git", github_owner(), github_repo())}
                                }
                            }
                        }
                    }
                }

                // Claude Settings
                div { class: "mb-6",
                    h2 { class: "text-xl font-semibold mb-4 text-gray-700", "Claude Settings" }
                    
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div { class: "md:col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "Claude API Key ",
                                span { class: "text-xs text-gray-500", "(Required for Claude Code)" }
                            }
                            div { class: "relative",
                                input {
                                    r#type: if show_api_key() { "text" } else { "password" },
                                    class: "w-full px-3 py-2 pr-10 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    value: "{claude_api_key}",
                                    placeholder: "sk-ant-...",
                                    oninput: move |e| claude_api_key.set(e.value())
                                }
                                button {
                                    r#type: "button",
                                    class: "absolute right-2 top-2 text-gray-500 hover:text-gray-700",
                                    onclick: move |_| show_api_key.set(!show_api_key()),
                                    if show_api_key() { "👁" } else { "🔒" }
                                }
                            }
                        }
                        
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Claude Model" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{claude_model}",
                                onchange: move |e| claude_model.set(e.value()),
                                option { value: "claude-3-opus-20240229", "Claude 3 Opus" }
                                option { value: "claude-3-sonnet-20240229", "Claude 3 Sonnet" }
                                option { value: "claude-3-haiku-20240307", "Claude 3 Haiku" }
                            }
                        }
                        
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", 
                                "Max Session Duration (minutes)" 
                            }
                            input {
                                r#type: "number",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{max_session_duration}",
                                min: "5",
                                max: "240",
                                oninput: move |e| {
                                    if let Ok(val) = e.value().parse::<i32>() {
                                        max_session_duration.set(val);
                                    }
                                }
                            }
                        }
                    }
                    
                    div { class: "mt-4",
                        label { class: "flex items-center",
                            input {
                                r#type: "checkbox",
                                class: "mr-2",
                                checked: auto_create_pr(),
                                onchange: move |e| auto_create_pr.set(e.checked())
                            }
                            span { class: "text-sm text-gray-700", "Automatically create pull requests" }
                        }
                    }
                }

                // Save Button and Status
                div { class: "flex items-center justify-between mt-6 pt-6 border-t border-gray-200",
                    div { class: "text-sm",
                        if !save_status().is_empty() {
                            span { class: if save_status().starts_with("✅") { "text-green-600" } else { "text-red-600" },
                                "{save_status}"
                            }
                        }
                    }
                    
                    button {
                        r#type: "button",
                        class: "px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500",
                        onclick: move |_| {
                            save_status.set("Saving...".to_string());
                            let repo = repository.clone();
                            
                            spawn(async move {
                                let mut cfg = config().unwrap_or_else(|| {
                                    ClaudeCodeConfig::new(
                                        github_repo(),
                                        github_owner(),
                                    )
                                });

                                // Update config fields
                                cfg.github_repo = github_repo();
                                cfg.github_owner = github_owner();
                                cfg.github_token = if github_token().is_empty() { None } else { Some(github_token()) };
                                cfg.claude_api_key = if claude_api_key().is_empty() { None } else { Some(claude_api_key()) };
                                cfg.default_base_branch = default_base_branch();
                                cfg.auto_create_pr = auto_create_pr();
                                cfg.working_directory = if working_directory().is_empty() { None } else { Some(working_directory()) };
                                cfg.claude_model = claude_model();
                                cfg.max_session_duration_minutes = max_session_duration();
                                cfg.updated_at = chrono::Utc::now();

                                // Save to database
                                let result = if config().is_some() {
                                    repo.claude_code.update_config(&cfg).await
                                } else {
                                    repo.claude_code.create_config(&cfg).await
                                };

                                match result {
                                    Ok(_) => {
                                        config.set(Some(cfg));
                                        save_status.set("✅ Configuration saved successfully!".to_string());
                                    }
                                    Err(e) => {
                                        save_status.set(format!("❌ Error saving: {}", e));
                                    }
                                }
                            });
                        },
                        "Save Configuration"
                    }
                }
            }

            // Info Box
            div { class: "mt-6 bg-blue-50 border border-blue-200 rounded-lg p-4",
                h3 { class: "text-sm font-semibold text-blue-900 mb-2", "Configuration Info" }
                ul { class: "text-sm text-blue-800 space-y-1",
                    li { "• Each task execution creates an isolated workspace folder" }
                    li { "• Repositories are cloned fresh for each task" }
                    li { "• GitHub token is only needed for private repositories" }
                    li { "• Claude API key is required to launch Claude Code sessions" }
                    li { "• Workspace folders are named: task-[id_short]-[title_slug]" }
                }
            }
        }
    }
}