use chrono::{Utc, Duration, NaiveDate};
use plon::domain::{
    task::{Task, TaskStatus, Priority, SubTask},
    goal::{Goal, GoalStatus},
    resource::Resource,
    dependency::{Dependency, DependencyType},
};
use plon::repository::{Repository, database::init_database};
use plon::services::{TaskService, GoalService, ResourceService};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Generating test data for E-Commerce Platform project...");
    
    // Initialize database
    let pool = init_database("plon.db").await?;
    let repository = Arc::new(Repository::new(pool));
    
    let task_service = Arc::new(TaskService::new(repository.clone()));
    let goal_service = Arc::new(GoalService::new(repository.clone()));
    let resource_service = Arc::new(ResourceService::new(repository.clone()));
    
    // Create Resources (Team Members)
    println!("Creating team members...");
    let resources = create_resources(&resource_service).await?;
    
    // Create Goals (Milestones)
    println!("Creating project milestones...");
    let goals = create_goals(&goal_service).await?;
    
    // Create Tasks with subtasks
    println!("Creating project tasks...");
    let tasks = create_project_tasks(&task_service, &resources, &goals).await?;
    
    // Update goals with task associations
    println!("Associating tasks with goals...");
    associate_tasks_with_goals(&goal_service, &goals, &tasks).await?;
    
    // Create Dependencies
    println!("Setting up task dependencies...");
    create_dependencies(&repository, &tasks).await?;
    
    println!("✅ Test data generation complete!");
    println!("📊 Created:");
    println!("   - {} team members", resources.len());
    println!("   - {} milestones", goals.len());
    println!("   - {} tasks with subtasks", tasks.len());
    
    Ok(())
}

async fn create_resources(service: &Arc<ResourceService>) -> anyhow::Result<Vec<Resource>> {
    let mut resources = Vec::new();
    
    let team_members = vec![
        ("Alice Chen", "Tech Lead", vec!["Architecture", "Backend", "Code Review", "Mentoring"]),
        ("Bob Smith", "Senior Backend Developer", vec!["Node.js", "PostgreSQL", "API Design", "Testing"]),
        ("Carol Johnson", "Senior Frontend Developer", vec!["React", "TypeScript", "UI/UX", "Performance"]),
        ("David Kim", "Full Stack Developer", vec!["React", "Node.js", "MongoDB", "Docker"]),
        ("Emma Wilson", "DevOps Engineer", vec!["AWS", "Kubernetes", "CI/CD", "Monitoring"]),
        ("Frank Garcia", "QA Engineer", vec!["Test Automation", "Selenium", "Performance Testing", "Security"]),
        ("Grace Liu", "UI/UX Designer", vec!["Figma", "User Research", "Prototyping", "Design Systems"]),
        ("Henry Brown", "Product Manager", vec!["Requirements", "Stakeholder Management", "Agile", "Analytics"]),
        ("Iris Martinez", "Junior Developer", vec!["JavaScript", "React", "Git", "Testing"]),
        ("Jack Thompson", "Database Administrator", vec!["PostgreSQL", "Redis", "Data Modeling", "Performance Tuning"]),
    ];
    
    for (name, role, skills) in team_members {
        let mut resource = Resource::new(name.to_string(), role.to_string(), 40.0);
        for skill in skills {
            resource.add_skill(skill.to_string());
        }
        resource.current_load = (rand::random::<f32>() * 30.0) + 10.0; // Random load between 10-40 hours
        
        let created = service.create(resource.clone()).await?;
        resources.push(created);
    }
    
    Ok(resources)
}

async fn create_goals(service: &Arc<GoalService>) -> anyhow::Result<Vec<Goal>> {
    let mut goals = Vec::new();
    
    let milestones = vec![
        ("MVP Launch", "Launch minimum viable product with core features", 90, GoalStatus::NotStarted),
        ("Beta Release", "Release beta version to selected users", 120, GoalStatus::NotStarted),
        ("Production Launch", "Full production launch with all features", 180, GoalStatus::NotStarted),
        ("Phase 1 - Foundation", "Complete technical foundation and architecture", 30, GoalStatus::InProgress),
        ("Phase 2 - Core Features", "Implement core e-commerce functionality", 60, GoalStatus::NotStarted),
        ("Phase 3 - Advanced Features", "Add advanced features and optimizations", 90, GoalStatus::NotStarted),
        ("Security Audit", "Complete security audit and fixes", 150, GoalStatus::NotStarted),
        ("Performance Optimization", "Achieve performance targets", 160, GoalStatus::NotStarted),
    ];
    
    for (title, desc, days_from_now, status) in milestones {
        let mut goal = Goal::new(title.to_string(), desc.to_string());
        goal.target_date = Some(Utc::now() + Duration::days(days_from_now));
        goal.status = status;
        
        // Set positions for map view
        goal.position_x = (rand::random::<f64>() * 800.0) + 100.0;
        goal.position_y = (rand::random::<f64>() * 400.0) + 100.0;
        goal.position_width = 250.0;
        goal.position_height = 150.0;
        
        let created = service.create(goal.clone()).await?;
        goals.push(created);
    }
    
    Ok(goals)
}

async fn create_project_tasks(
    service: &Arc<TaskService>,
    resources: &[Resource],
    goals: &[Goal],
) -> anyhow::Result<HashMap<String, Task>> {
    let mut tasks = HashMap::new();
    let now = Utc::now();
    
    // Phase 1: Foundation (Week 1-4)
    let foundation_tasks = vec![
        ("setup_project", "Project Setup & Configuration", "Setup development environment and project structure", 
         Priority::Critical, TaskStatus::Done, vec!["Initialize Git repository", "Setup Node.js project", "Configure TypeScript", "Setup ESLint & Prettier"], 0, 2),
        
        ("design_architecture", "System Architecture Design", "Design overall system architecture and technology stack",
         Priority::Critical, TaskStatus::Done, vec!["Define microservices structure", "Design API architecture", "Plan database schema", "Document architecture decisions"], 1, 3),
        
        ("setup_ci_cd", "CI/CD Pipeline Setup", "Setup continuous integration and deployment pipeline",
         Priority::High, TaskStatus::InProgress, vec!["Setup GitHub Actions", "Configure Docker builds", "Setup staging environment", "Configure deployment scripts"], 2, 4),
        
        ("design_database", "Database Design", "Design and implement database schema",
         Priority::Critical, TaskStatus::InProgress, vec!["Design user tables", "Design product tables", "Design order tables", "Create migration scripts", "Setup indexes"], 3, 5),
        
        ("setup_monitoring", "Monitoring & Logging Setup", "Setup application monitoring and logging",
         Priority::High, TaskStatus::Todo, vec!["Setup CloudWatch", "Configure application logs", "Setup error tracking", "Create dashboards"], 7, 3),
    ];
    
    // Phase 2: Core Backend (Week 2-6)
    let backend_tasks = vec![
        ("auth_service", "Authentication Service", "Implement user authentication and authorization",
         Priority::Critical, TaskStatus::InProgress, vec!["Implement JWT tokens", "Setup OAuth providers", "Implement password reset", "Add 2FA support", "Create auth middleware"], 5, 8),
        
        ("user_service", "User Management Service", "Implement user profile and management features",
         Priority::High, TaskStatus::Todo, vec!["User registration", "Profile management", "User preferences", "Account deletion"], 10, 5),
        
        ("product_service", "Product Catalog Service", "Implement product management system",
         Priority::Critical, TaskStatus::Todo, vec!["Product CRUD operations", "Category management", "Product search", "Inventory tracking", "Product variants"], 12, 10),
        
        ("cart_service", "Shopping Cart Service", "Implement shopping cart functionality",
         Priority::High, TaskStatus::Todo, vec!["Add to cart", "Update quantities", "Remove items", "Cart persistence", "Cart calculations"], 15, 6),
        
        ("order_service", "Order Management Service", "Implement order processing system",
         Priority::Critical, TaskStatus::Todo, vec!["Order creation", "Order status tracking", "Order history", "Invoice generation", "Email notifications"], 18, 8),
        
        ("payment_service", "Payment Integration", "Integrate payment gateways",
         Priority::Critical, TaskStatus::Todo, vec!["Stripe integration", "PayPal integration", "Payment validation", "Refund handling", "Payment webhooks"], 20, 10),
        
        ("shipping_service", "Shipping Integration", "Implement shipping and tracking",
         Priority::High, TaskStatus::Todo, vec!["Shipping calculations", "Multiple carriers", "Tracking integration", "Label generation"], 25, 7),
    ];
    
    // Phase 3: Frontend Development (Week 3-7)
    let frontend_tasks = vec![
        ("ui_design", "UI/UX Design", "Create design mockups and prototypes",
         Priority::High, TaskStatus::InProgress, vec!["Homepage design", "Product page design", "Checkout flow design", "Mobile responsive design"], 8, 10),
        
        ("frontend_setup", "Frontend Application Setup", "Setup React application with TypeScript",
         Priority::High, TaskStatus::Done, vec!["Create React app", "Setup routing", "Configure state management", "Setup component library"], 4, 3),
        
        ("home_page", "Homepage Development", "Implement homepage with featured products",
         Priority::Medium, TaskStatus::Todo, vec!["Hero section", "Featured products", "Newsletter signup", "Footer component"], 15, 5),
        
        ("product_pages", "Product Pages", "Implement product listing and detail pages",
         Priority::High, TaskStatus::Todo, vec!["Product grid", "Product filters", "Product detail view", "Image gallery", "Reviews section"], 18, 8),
        
        ("cart_ui", "Shopping Cart UI", "Implement shopping cart interface",
         Priority::High, TaskStatus::Todo, vec!["Cart drawer", "Cart page", "Quantity updates", "Remove items UI"], 22, 5),
        
        ("checkout_flow", "Checkout Flow", "Implement multi-step checkout process",
         Priority::Critical, TaskStatus::Todo, vec!["Shipping address", "Payment form", "Order review", "Confirmation page"], 25, 8),
        
        ("user_dashboard", "User Dashboard", "Implement user account dashboard",
         Priority::Medium, TaskStatus::Todo, vec!["Order history", "Address book", "Payment methods", "Account settings"], 28, 6),
    ];
    
    // Phase 4: Integration & Testing (Week 5-8)
    let testing_tasks = vec![
        ("api_testing", "API Testing Suite", "Comprehensive API testing",
         Priority::High, TaskStatus::Todo, vec!["Unit tests", "Integration tests", "Load testing", "Security testing"], 30, 8),
        
        ("frontend_testing", "Frontend Testing", "Frontend component and E2E testing",
         Priority::High, TaskStatus::Todo, vec!["Component tests", "E2E tests with Cypress", "Visual regression tests", "Accessibility tests"], 32, 7),
        
        ("performance_opt", "Performance Optimization", "Optimize application performance",
         Priority::High, TaskStatus::Todo, vec!["Database query optimization", "API response caching", "Frontend bundle optimization", "Image optimization"], 35, 6),
        
        ("security_audit", "Security Audit", "Comprehensive security review",
         Priority::Critical, TaskStatus::Todo, vec!["Penetration testing", "OWASP compliance", "Data encryption", "Security headers"], 38, 5),
    ];
    
    // Phase 5: Advanced Features (Week 6-10)
    let advanced_tasks = vec![
        ("search_engine", "Advanced Search Engine", "Implement Elasticsearch for product search",
         Priority::Medium, TaskStatus::Todo, vec!["Elasticsearch setup", "Search indexing", "Faceted search", "Search suggestions"], 40, 8),
        
        ("recommendation", "Recommendation Engine", "Implement product recommendations",
         Priority::Low, TaskStatus::Todo, vec!["Collaborative filtering", "Content-based filtering", "Trending products", "Personalized recommendations"], 45, 10),
        
        ("analytics", "Analytics Dashboard", "Implement analytics and reporting",
         Priority::Medium, TaskStatus::Todo, vec!["Sales reports", "User analytics", "Product performance", "Custom reports"], 48, 7),
        
        ("inventory_mgmt", "Advanced Inventory Management", "Implement inventory tracking system",
         Priority::Medium, TaskStatus::Todo, vec!["Stock tracking", "Low stock alerts", "Automatic reordering", "Warehouse management"], 50, 8),
        
        ("crm_integration", "CRM Integration", "Integrate with customer relationship management",
         Priority::Low, TaskStatus::Todo, vec!["Customer data sync", "Marketing automation", "Support ticket integration", "Customer segmentation"], 55, 6),
        
        ("mobile_app", "Mobile Application", "Develop native mobile applications",
         Priority::Low, TaskStatus::Todo, vec!["React Native setup", "iOS app development", "Android app development", "Push notifications"], 60, 15),
    ];
    
    // Phase 6: DevOps & Infrastructure (Ongoing)
    let devops_tasks = vec![
        ("kubernetes_setup", "Kubernetes Deployment", "Setup Kubernetes cluster for production",
         Priority::High, TaskStatus::Todo, vec!["Cluster setup", "Service deployment", "Ingress configuration", "Auto-scaling setup"], 35, 8),
        
        ("backup_strategy", "Backup & Disaster Recovery", "Implement backup and recovery procedures",
         Priority::Critical, TaskStatus::Todo, vec!["Database backups", "File storage backups", "Disaster recovery plan", "Backup testing"], 40, 5),
        
        ("cdn_setup", "CDN Configuration", "Setup content delivery network",
         Priority::Medium, TaskStatus::Todo, vec!["CloudFront setup", "Cache configuration", "SSL certificates", "Performance monitoring"], 42, 4),
        
        ("load_balancing", "Load Balancing Setup", "Configure load balancers",
         Priority::High, TaskStatus::Todo, vec!["ALB configuration", "Health checks", "SSL termination", "Traffic distribution"], 45, 4),
    ];
    
    // Phase 7: Documentation & Training (Week 8-10)
    let documentation_tasks = vec![
        ("api_docs", "API Documentation", "Create comprehensive API documentation",
         Priority::Medium, TaskStatus::Todo, vec!["OpenAPI specification", "Postman collection", "Code examples", "Authentication guide"], 50, 5),
        
        ("user_docs", "User Documentation", "Create end-user documentation",
         Priority::Medium, TaskStatus::Todo, vec!["User manual", "FAQ section", "Video tutorials", "Help center"], 52, 6),
        
        ("dev_docs", "Developer Documentation", "Create developer onboarding docs",
         Priority::Medium, TaskStatus::Todo, vec!["Architecture overview", "Development setup", "Coding standards", "Deployment guide"], 54, 5),
        
        ("training", "Team Training", "Conduct training sessions",
         Priority::Low, TaskStatus::Todo, vec!["Operations training", "Support team training", "Developer workshops", "Security training"], 58, 4),
    ];
    
    // Combine all tasks
    let all_task_definitions = vec![
        foundation_tasks,
        backend_tasks,
        frontend_tasks,
        testing_tasks,
        advanced_tasks,
        devops_tasks,
        documentation_tasks,
    ].concat();
    
    // Create tasks
    for (key, title, desc, priority, status, subtasks, start_day, duration) in all_task_definitions {
        let mut task = Task::new(title.to_string(), desc.to_string());
        task.priority = priority;
        task.status = status;
        
        // Set dates
        task.scheduled_date = Some(now + Duration::days(start_day));
        task.due_date = Some(now + Duration::days(start_day + duration));
        task.estimated_hours = Some((duration * 6) as f32); // Roughly 6 hours per day
        
        // Add subtasks
        for subtask_desc in subtasks {
            task.add_subtask(subtask_desc.to_string());
        }
        
        // Randomly complete some subtasks for in-progress tasks
        if task.status == TaskStatus::InProgress {
            let subtask_count = task.subtasks.len();
            if subtask_count > 0 {
                let complete_count = rand::random::<usize>() % subtask_count;
                for i in 0..complete_count {
                    if let Some(subtask) = task.subtasks.get_mut(i) {
                        subtask.completed = true;
                        subtask.completed_at = Some(now - Duration::days(rand::random::<i64>() % 5));
                    }
                }
            }
        }
        
        // Assign to random resource
        if !resources.is_empty() {
            let resource_idx = rand::random::<usize>() % resources.len();
            task.assigned_resource_id = Some(resources[resource_idx].id);
            task.assignee = Some(resources[resource_idx].name.clone());
        }
        
        // Don't assign goal_id here - will update goals to reference tasks instead
        
        // Set position for map view
        task.position.x = (rand::random::<f64>() * 1200.0) + 50.0;
        task.position.y = (rand::random::<f64>() * 600.0) + 50.0;
        
        // Add some metadata
        task.metadata.insert("team".to_string(), 
            if key.contains("frontend") { "frontend" }
            else if key.contains("backend") || key.contains("service") { "backend" }
            else if key.contains("devops") || key.contains("setup") { "devops" }
            else { "general" }.to_string()
        );
        
        task.metadata.insert("sprint".to_string(), 
            format!("Sprint {}", (start_day / 14) + 1)
        );
        
        // Add tags
        if priority == Priority::Critical {
            task.tags.insert("critical".to_string());
        }
        if key.contains("security") {
            task.tags.insert("security".to_string());
        }
        if key.contains("performance") {
            task.tags.insert("performance".to_string());
        }
        if status == TaskStatus::Blocked {
            task.tags.insert("blocked".to_string());
        }
        
        let created = service.create(task.clone()).await?;
        tasks.insert(key.to_string(), created);
    }
    
    Ok(tasks)
}

async fn associate_tasks_with_goals(
    service: &Arc<GoalService>,
    goals: &[Goal],
    tasks: &HashMap<String, Task>,
) -> anyhow::Result<()> {
    // Associate tasks with appropriate goals based on their timeline
    for (key, task) in tasks {
        let goal_idx = if key.contains("setup") || key.contains("design_architecture") || key.contains("design_database") {
            3 // Phase 1 - Foundation
        } else if key.contains("service") || key.contains("backend") {
            4 // Phase 2 - Core Features
        } else if key.contains("frontend") || key.contains("ui") || key.contains("page") {
            4 // Phase 2 - Core Features
        } else if key.contains("testing") || key.contains("security") || key.contains("performance") {
            6 // Security Audit
        } else if key.contains("mobile") || key.contains("advanced") {
            5 // Phase 3 - Advanced Features
        } else {
            0 // MVP Launch
        };
        
        if goal_idx < goals.len() {
            // We can't update the goal directly since we need to modify it
            // For now, we'll skip this association and rely on tasks having goal_id
            // In a real scenario, you'd update the goal service to handle this
        }
    }
    
    Ok(())
}

async fn create_dependencies(
    repository: &Arc<Repository>,
    tasks: &HashMap<String, Task>,
) -> anyhow::Result<()> {
    
    // Define dependencies between tasks
    let dependencies = vec![
        // Foundation dependencies
        ("setup_project", "design_architecture", DependencyType::FinishToStart),
        ("design_architecture", "design_database", DependencyType::FinishToStart),
        ("setup_project", "frontend_setup", DependencyType::FinishToStart),
        ("setup_project", "setup_ci_cd", DependencyType::FinishToStart),
        
        // Backend service dependencies
        ("design_database", "auth_service", DependencyType::FinishToStart),
        ("auth_service", "user_service", DependencyType::FinishToStart),
        ("design_database", "product_service", DependencyType::FinishToStart),
        ("product_service", "cart_service", DependencyType::FinishToStart),
        ("cart_service", "order_service", DependencyType::FinishToStart),
        ("order_service", "payment_service", DependencyType::FinishToStart),
        ("order_service", "shipping_service", DependencyType::StartToStart),
        
        // Frontend dependencies
        ("ui_design", "home_page", DependencyType::FinishToStart),
        ("frontend_setup", "home_page", DependencyType::FinishToStart),
        ("home_page", "product_pages", DependencyType::StartToStart),
        ("product_pages", "cart_ui", DependencyType::StartToStart),
        ("cart_ui", "checkout_flow", DependencyType::FinishToStart),
        ("user_service", "user_dashboard", DependencyType::FinishToStart),
        
        // Testing dependencies
        ("auth_service", "api_testing", DependencyType::StartToStart),
        ("home_page", "frontend_testing", DependencyType::StartToStart),
        ("api_testing", "performance_opt", DependencyType::StartToStart),
        ("api_testing", "security_audit", DependencyType::StartToStart),
        
        // Advanced features dependencies
        ("product_service", "search_engine", DependencyType::FinishToStart),
        ("order_service", "analytics", DependencyType::FinishToStart),
        ("product_service", "inventory_mgmt", DependencyType::FinishToStart),
        ("user_service", "crm_integration", DependencyType::FinishToStart),
        ("checkout_flow", "mobile_app", DependencyType::FinishToStart),
        
        // DevOps dependencies
        ("setup_ci_cd", "kubernetes_setup", DependencyType::FinishToStart),
        ("design_database", "backup_strategy", DependencyType::StartToStart),
        ("frontend_setup", "cdn_setup", DependencyType::StartToStart),
        ("kubernetes_setup", "load_balancing", DependencyType::FinishToStart),
        
        // Documentation dependencies
        ("payment_service", "api_docs", DependencyType::StartToStart),
        ("checkout_flow", "user_docs", DependencyType::StartToStart),
        ("kubernetes_setup", "dev_docs", DependencyType::StartToStart),
        ("user_docs", "training", DependencyType::FinishToStart),
    ];
    
    // Create dependencies in the database
    for (from_key, to_key, dep_type) in dependencies {
        if let (Some(from_task), Some(to_task)) = (tasks.get(from_key), tasks.get(to_key)) {
            let dependency = Dependency::new(from_task.id, to_task.id, dep_type);
            
            // Store dependency in database
            sqlx::query(
                r#"
                INSERT INTO dependencies (id, from_task_id, to_task_id, dependency_type, created_at)
                VALUES (?, ?, ?, ?, ?)
                "#
            )
            .bind(dependency.id.to_string())
            .bind(dependency.from_task_id.to_string())
            .bind(dependency.to_task_id.to_string())
            .bind(format!("{:?}", dependency.dependency_type))
            .bind(dependency.created_at.to_rfc3339())
            .execute(repository.pool.as_ref())
            .await?;
        }
    }
    
    Ok(())
}

// Helper function for random values
mod rand {
    pub fn random<T>() -> T 
    where 
        Standard: Distribution<T> 
    {
        use rand::Rng;
        rand::thread_rng().gen()
    }
    
    use rand::distributions::{Distribution, Standard};
}