use dioxus::prelude::*;
use dioxus::prelude::UnboundedReceiver;
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use crate::domain::goal::{Goal, GoalStatus};
use crate::domain::task::{Task, TaskStatus};
use std::sync::Arc;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub fn GoalsView() -> Element {
    let repository = use_context::<Arc<Repository>>();
    let mut goals = use_signal(|| Vec::<Goal>::new());
    let mut tasks = use_signal(|| Vec::<Task>::new());
    let mut loading = use_signal(|| true);
    let mut error_message = use_signal(String::new);
    let mut selected_goal_id = use_signal(|| None::<Uuid>);
    let mut editing_goal = use_signal(|| None::<Goal>);
    let mut creating_goal = use_signal(|| false);
    let mut show_task_modal = use_signal(|| false);
    let mut selected_goal_for_tasks = use_signal(|| None::<Uuid>);

    use_coroutine({
        let repository = repository.clone();
        let mut goals = goals.clone();
        let mut tasks = tasks.clone();
        let mut loading = loading.clone();
        let mut error_message = error_message.clone();

        move |_: UnboundedReceiver<()>| async move {
            loading.set(true);
            
            match repository.goals.list_all().await {
                Ok(fetched_goals) => {
                    goals.set(fetched_goals);
                }
                Err(e) => {
                    error_message.set(format!("Failed to load goals: {}", e));
                }
            }

            match repository.tasks.list(TaskFilters::default()).await {
                Ok(fetched_tasks) => {
                    tasks.set(fetched_tasks);
                }
                Err(e) => {
                    error_message.set(format!("Failed to load tasks: {}", e));
                }
            }

            loading.set(false);
        }
    });

    let handle_create_goal = {
        let repository = repository.clone();
        let mut goals = goals.clone();
        let mut creating_goal = creating_goal.clone();
        let mut error_message = error_message.clone();

        move |(title, description): (String, String)| {
            let repository = repository.clone();
            let mut goals = goals.clone();
            let mut creating_goal = creating_goal.clone();
            let mut error_message = error_message.clone();

            spawn(async move {
                let new_goal = Goal::new(title, description);
                match repository.goals.create(&new_goal).await {
                    Ok(_) => {
                        let mut current_goals = goals();
                        current_goals.push(new_goal);
                        goals.set(current_goals);
                        creating_goal.set(false);
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to create goal: {}", e));
                    }
                }
            });
        }
    };

    let handle_update_goal = {
        let repository = repository.clone();
        let mut goals = goals.clone();
        let mut editing_goal = editing_goal.clone();
        let mut error_message = error_message.clone();

        move |goal: Goal| {
            let repository = repository.clone();
            let mut goals = goals.clone();
            let mut editing_goal = editing_goal.clone();
            let mut error_message = error_message.clone();

            spawn(async move {
                match repository.goals.update(&goal).await {
                    Ok(_) => {
                        let mut current_goals = goals();
                        if let Some(idx) = current_goals.iter().position(|g| g.id == goal.id) {
                            current_goals[idx] = goal;
                            goals.set(current_goals);
                        }
                        editing_goal.set(None);
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to update goal: {}", e));
                    }
                }
            });
        }
    };

    let handle_delete_goal = {
        let repository = repository.clone();
        let mut goals = goals.clone();
        let mut error_message = error_message.clone();

        move |goal_id: Uuid| {
            let repository = repository.clone();
            let mut goals = goals.clone();
            let mut error_message = error_message.clone();

            spawn(async move {
                match repository.goals.delete(goal_id).await {
                    Ok(_) => {
                        let current_goals = goals();
                        let filtered_goals: Vec<Goal> = current_goals.into_iter()
                            .filter(|g| g.id != goal_id)
                            .collect();
                        goals.set(filtered_goals);
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to delete goal: {}", e));
                    }
                }
            });
        }
    };

    let handle_toggle_task = {
        let repository = repository.clone();
        let mut goals = goals.clone();
        let mut tasks = tasks.clone();
        let mut error_message = error_message.clone();

        move |goal_id: Uuid, task_id: Uuid, add: bool| {
            let repository = repository.clone();
            let mut goals = goals.clone();
            let mut tasks = tasks.clone();
            let mut error_message = error_message.clone();

            spawn(async move {
                let mut current_goals = goals();
                if let Some(goal) = current_goals.iter_mut().find(|g| g.id == goal_id) {
                    if add {
                        goal.add_task(task_id);
                        
                        if let Some(task) = tasks().iter_mut().find(|t| t.id == task_id) {
                            task.goal_id = Some(goal_id);
                            let _ = repository.tasks.update(task).await;
                        }
                    } else {
                        goal.remove_task(&task_id);
                        
                        if let Some(task) = tasks().iter_mut().find(|t| t.id == task_id) {
                            task.goal_id = None;
                            let _ = repository.tasks.update(task).await;
                        }
                    }
                    
                    goal.update_progress(&tasks());
                    
                    match repository.goals.update(goal).await {
                        Ok(_) => {
                            goals.set(current_goals);
                        }
                        Err(e) => {
                            error_message.set(format!("Failed to update goal tasks: {}", e));
                        }
                    }
                }
            });
        }
    };

    rsx! {
        div {
            class: "goals-view p-6",
            
            // Header
            div {
                class: "mb-6 flex justify-between items-center",
                h1 { class: "text-3xl font-bold text-gray-800", "Goals" }
                button {
                    class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                    onclick: move |_| creating_goal.set(true),
                    "➕ New Goal"
                }
            }

            // Error message
            if !error_message().is_empty() {
                div {
                    class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4",
                    {error_message()}
                }
            }

            // Loading state
            if *loading.read() {
                div {
                    class: "flex justify-center items-center h-64",
                    div { class: "text-lg text-gray-600", "Loading goals..." }
                }
            }

            // Goals grid
            if !*loading.read() {
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                    
                    for goal in goals() {
                        GoalCard {
                            goal: goal.clone(),
                            tasks: tasks(),
                            on_edit: {
                                let mut editing_goal = editing_goal.clone();
                                move |g| editing_goal.set(Some(g))
                            },
                            on_delete: {
                                let handle_delete_goal = handle_delete_goal.clone();
                                move |id| (handle_delete_goal)(id)
                            },
                            on_manage_tasks: {
                                let mut selected_goal_for_tasks = selected_goal_for_tasks.clone();
                                let mut show_task_modal = show_task_modal.clone();
                                move |id| {
                                    selected_goal_for_tasks.set(Some(id));
                                    show_task_modal.set(true);
                                }
                            }
                        }
                    }
                }
            }

            // Empty state
            if !*loading.read() && goals().is_empty() {
                div {
                    class: "text-center py-12",
                    p { class: "text-gray-500 text-lg mb-4", "No goals yet. Create your first goal to get started!" }
                    button {
                        class: "px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600",
                        onclick: move |_| creating_goal.set(true),
                        "Create First Goal"
                    }
                }
            }

            // Create Goal Modal
            if *creating_goal.read() {
                CreateGoalModal {
                    on_create: handle_create_goal.clone(),
                    on_cancel: {
                        let mut creating_goal = creating_goal.clone();
                        move |_| creating_goal.set(false)
                    }
                }
            }

            // Edit Goal Modal
            if let Some(goal) = editing_goal() {
                EditGoalModal {
                    goal: goal,
                    on_save: handle_update_goal.clone(),
                    on_cancel: {
                        let mut editing_goal = editing_goal.clone();
                        move |_| editing_goal.set(None)
                    }
                }
            }

            // Manage Tasks Modal
            if *show_task_modal.read() && selected_goal_for_tasks().is_some() {
                if let Some(goal_id) = selected_goal_for_tasks() {
                    if let Some(goal) = goals().iter().find(|g| g.id == goal_id) {
                        ManageTasksModal {
                            goal: goal.clone(),
                            all_tasks: tasks(),
                            on_toggle_task: {
                                let handle_toggle_task = handle_toggle_task.clone();
                                move |(task_id, add)| (handle_toggle_task)(goal_id, task_id, add)
                            },
                            on_close: {
                                let mut show_task_modal = show_task_modal.clone();
                                move |_| show_task_modal.set(false)
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GoalCard(
    goal: Goal,
    tasks: Vec<Task>,
    on_edit: EventHandler<Goal>,
    on_delete: EventHandler<Uuid>,
    on_manage_tasks: EventHandler<Uuid>
) -> Element {
    let task_count = goal.task_ids.len();
    let completed_tasks = tasks.iter()
        .filter(|t| goal.task_ids.contains(&t.id) && t.status == TaskStatus::Done)
        .count();
    
    let status_color = match goal.status {
        GoalStatus::NotStarted => "bg-gray-100 text-gray-800",
        GoalStatus::Active | GoalStatus::InProgress => "bg-blue-100 text-blue-800",
        GoalStatus::OnHold => "bg-yellow-100 text-yellow-800",
        GoalStatus::AtRisk => "bg-red-100 text-red-800",
        GoalStatus::Completed => "bg-green-100 text-green-800",
        GoalStatus::Cancelled => "bg-gray-100 text-gray-500",
    };

    let progress_percentage = if task_count > 0 {
        (completed_tasks as f32 / task_count as f32 * 100.0) as u32
    } else {
        0
    };

    rsx! {
        div {
            class: "bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow",
            style: format!("border-left: 4px solid {}", goal.color),
            
            // Header
            div {
                class: "flex justify-between items-start mb-4",
                h3 { 
                    class: "text-xl font-semibold text-gray-800 flex-1",
                    {goal.title.clone()}
                }
                div {
                    class: "flex gap-2",
                    button {
                        class: "text-gray-500 hover:text-blue-600",
                        onclick: {
                            let goal_clone = goal.clone();
                            move |_| on_edit.call(goal_clone.clone())
                        },
                        "✏️"
                    }
                    button {
                        class: "text-gray-500 hover:text-red-600",
                        onclick: {
                            let goal_id = goal.id;
                            move |_| on_delete.call(goal_id)
                        },
                        "🗑️"
                    }
                }
            }

            // Status badge
            div {
                class: format!("inline-block px-3 py-1 rounded-full text-sm font-medium mb-3 {}", status_color),
                {format!("{:?}", goal.status)}
            }

            // Description
            if !goal.description.is_empty() {
                p {
                    class: "text-gray-600 mb-4 line-clamp-3",
                    {goal.description.clone()}
                }
            }

            // Progress bar
            div {
                class: "mb-4",
                div {
                    class: "flex justify-between text-sm text-gray-600 mb-1",
                    span { "Progress" }
                    span { {format!("{}%", progress_percentage)} }
                }
                div {
                    class: "w-full bg-gray-200 rounded-full h-2",
                    div {
                        class: "bg-blue-500 h-2 rounded-full transition-all",
                        style: format!("width: {}%", progress_percentage)
                    }
                }
            }

            // Tasks info
            div {
                class: "flex justify-between items-center mb-4",
                div {
                    class: "text-sm text-gray-600",
                    {format!("{} / {} tasks completed", completed_tasks, task_count)}
                }
                button {
                    class: "text-sm px-3 py-1 bg-blue-50 text-blue-600 rounded hover:bg-blue-100",
                    onclick: {
                        let goal_id = goal.id;
                        move |_| on_manage_tasks.call(goal_id)
                    },
                    "Manage Tasks"
                }
            }

            // Target date
            if let Some(target) = goal.target_date {
                div {
                    class: "text-sm text-gray-500",
                    "🎯 Target: {target.format(\"%b %d, %Y\")}"
                }
            }

            // Risk indicator
            if goal.is_at_risk() {
                div {
                    class: "mt-2 text-sm text-red-600 font-medium",
                    "⚠️ At Risk - Less than 7 days remaining"
                }
            }
        }
    }
}

#[component]
fn CreateGoalModal(
    on_create: EventHandler<(String, String)>,
    on_cancel: EventHandler<()>
) -> Element {
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),
            
            div {
                class: "bg-white rounded-lg p-6 w-full max-w-md",
                onclick: move |e| e.stop_propagation(),
                
                h2 { class: "text-2xl font-bold mb-4", "Create New Goal" }
                
                input {
                    class: "w-full px-3 py-2 border rounded mb-4",
                    placeholder: "Goal title",
                    value: "{title}",
                    oninput: move |e| title.set(e.value())
                }
                
                textarea {
                    class: "w-full px-3 py-2 border rounded mb-4 h-32",
                    placeholder: "Goal description",
                    value: "{description}",
                    oninput: move |e| description.set(e.value())
                }
                
                div {
                    class: "flex justify-end gap-2",
                    button {
                        class: "px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                        onclick: move |_| {
                            if !title().is_empty() {
                                on_create.call((title(), description()));
                            }
                        },
                        "Create"
                    }
                }
            }
        }
    }
}

#[component]
fn EditGoalModal(
    goal: Goal,
    on_save: EventHandler<Goal>,
    on_cancel: EventHandler<()>
) -> Element {
    let mut edited_goal = use_signal(|| goal.clone());

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),
            
            div {
                class: "bg-white rounded-lg p-6 w-full max-w-md",
                onclick: move |e| e.stop_propagation(),
                
                h2 { class: "text-2xl font-bold mb-4", "Edit Goal" }
                
                input {
                    class: "w-full px-3 py-2 border rounded mb-4",
                    placeholder: "Goal title",
                    value: "{edited_goal().title}",
                    oninput: move |e| {
                        let mut g = edited_goal();
                        g.title = e.value();
                        edited_goal.set(g);
                    }
                }
                
                textarea {
                    class: "w-full px-3 py-2 border rounded mb-4 h-32",
                    placeholder: "Goal description",
                    value: "{edited_goal().description}",
                    oninput: move |e| {
                        let mut g = edited_goal();
                        g.description = e.value();
                        edited_goal.set(g);
                    }
                }
                
                select {
                    class: "w-full px-3 py-2 border rounded mb-4",
                    value: format!("{:?}", edited_goal().status),
                    onchange: move |e| {
                        let mut g = edited_goal();
                        g.status = match e.value().as_str() {
                            "NotStarted" => GoalStatus::NotStarted,
                            "Active" => GoalStatus::Active,
                            "InProgress" => GoalStatus::InProgress,
                            "OnHold" => GoalStatus::OnHold,
                            "AtRisk" => GoalStatus::AtRisk,
                            "Completed" => GoalStatus::Completed,
                            "Cancelled" => GoalStatus::Cancelled,
                            _ => GoalStatus::NotStarted,
                        };
                        edited_goal.set(g);
                    },
                    option { value: "NotStarted", "Not Started" }
                    option { value: "Active", "Active" }
                    option { value: "InProgress", "In Progress" }
                    option { value: "OnHold", "On Hold" }
                    option { value: "AtRisk", "At Risk" }
                    option { value: "Completed", "Completed" }
                    option { value: "Cancelled", "Cancelled" }
                }
                
                div {
                    class: "flex justify-end gap-2",
                    button {
                        class: "px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                        onclick: move |_| on_save.call(edited_goal()),
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
fn ManageTasksModal(
    goal: Goal,
    all_tasks: Vec<Task>,
    on_toggle_task: EventHandler<(Uuid, bool)>,
    on_close: EventHandler<()>
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_close.call(()),
            
            div {
                class: "bg-white rounded-lg p-6 w-full max-w-2xl max-h-96 overflow-y-auto",
                onclick: move |e| e.stop_propagation(),
                
                h2 { 
                    class: "text-2xl font-bold mb-4", 
                    {format!("Manage Tasks for: {}", goal.title)}
                }
                
                div {
                    class: "space-y-2",
                    for task in all_tasks {
                        div {
                            class: "flex items-center justify-between p-2 hover:bg-gray-50 rounded",
                            div {
                                class: "flex items-center gap-3",
                                input {
                                    r#type: "checkbox",
                                    class: "w-4 h-4",
                                    checked: goal.task_ids.contains(&task.id),
                                    onchange: {
                                        let task_id = task.id;
                                        let is_assigned = goal.task_ids.contains(&task_id);
                                        let on_toggle_task = on_toggle_task.clone();
                                        move |_| {
                                            on_toggle_task.call((task_id, !is_assigned))
                                        }
                                    }
                                }
                                div {
                                    span { class: "font-medium", {task.title.clone()} }
                                    span { 
                                        class: "ml-2 text-sm text-gray-500",
                                        {format!("({})", match task.status {
                                            TaskStatus::Todo => "Todo",
                                            TaskStatus::InProgress => "In Progress",
                                            TaskStatus::Done => "Done",
                                            TaskStatus::Blocked => "Blocked",
                                            TaskStatus::Review => "Review",
                                            TaskStatus::Cancelled => "Cancelled",
                                        })}
                                    }
                                }
                            }
                        }
                    }
                }
                
                div {
                    class: "flex justify-end mt-4",
                    button {
                        class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                        onclick: move |_| on_close.call(()),
                        "Done"
                    }
                }
            }
        }
    }
}