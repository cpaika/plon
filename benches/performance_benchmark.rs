use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use plon::domain::{
    dependency::{Dependency, DependencyGraph, DependencyType},
    task::{Position, Priority, Task, TaskStatus},
};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use uuid::Uuid;

fn create_task_with_position(x: f64, y: f64) -> Task {
    let mut rng = rand::thread_rng();
    
    Task {
        id: Uuid::new_v4(),
        title: format!("Task at ({:.0}, {:.0})", x, y),
        description: format!("Description for task at position ({:.0}, {:.0})", x, y),
        status: match rng.gen_range(0..5) {
            0 => TaskStatus::Todo,
            1 => TaskStatus::InProgress,
            2 => TaskStatus::Blocked,
            3 => TaskStatus::Review,
            _ => TaskStatus::Done,
        },
        priority: match rng.gen_range(0..4) {
            0 => Priority::Low,
            1 => Priority::Medium,
            2 => Priority::High,
            _ => Priority::Critical,
        },
        metadata: HashMap::new(),
        tags: (0..rng.gen_range(0..5))
            .map(|i| format!("tag{}", i))
            .collect(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: if rng.gen_bool(0.5) {
            Some(Utc::now() + chrono::Duration::days(rng.gen_range(1..30)))
        } else {
            None
        },
        scheduled_date: None,
        completed_at: None,
        estimated_hours: Some(rng.gen_range(1.0..41.0)),
        actual_hours: None,
        assigned_resource_id: if rng.gen_bool(0.5) {
            Some(Uuid::new_v4())
        } else {
            None
        },
        goal_id: if rng.gen_bool(0.5) {
            Some(Uuid::new_v4())
        } else {
            None
        },
        parent_task_id: None,
        position: Position { x, y },
        subtasks: Vec::new(),
    }
}

fn create_nested_dependencies(tasks: &[Task], depth: usize, branching_factor: usize) -> Vec<Dependency> {
    let mut dependencies = Vec::new();
    let _tasks_per_level = branching_factor.pow(depth as u32);
    let total_tasks = tasks.len();
    
    if total_tasks < 2 {
        return dependencies;
    }
    
    for level in 0..depth {
        let level_start = branching_factor.pow(level as u32) - 1;
        let level_end = branching_factor.pow((level + 1) as u32) - 1;
        
        for i in level_start..level_end.min(total_tasks - 1) {
            for j in 0..branching_factor {
                let child_idx = (i + 1) * branching_factor + j;
                if child_idx < total_tasks {
                    let mut rng = rand::thread_rng();
                    dependencies.push(Dependency::new(
                        tasks[i].id,
                        tasks[child_idx].id,
                        match rng.gen_range(0..4) {
                            0 => DependencyType::StartToStart,
                            1 => DependencyType::FinishToFinish,
                            2 => DependencyType::StartToFinish,
                            _ => DependencyType::FinishToStart,
                        }
                    ));
                }
            }
        }
    }
    
    dependencies
}

fn bench_create_thousands_of_cards(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_cards");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    
    for count in [100, 500, 1000, 2500, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut tasks = Vec::with_capacity(count);
                    for i in 0..count {
                        let x = (i as f64 % 100.0) * 50.0;
                        let y = (i as f64 / 100.0) * 50.0;
                        tasks.push(create_task_with_position(x, y));
                    }
                    black_box(tasks)
                });
            },
        );
    }
    group.finish();
}

fn bench_nested_dependencies(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_dependencies");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));
    
    for (task_count, depth, branching) in [
        (100, 3, 3),
        (500, 4, 3),
        (1000, 5, 3),
        (2500, 5, 4),
        (5000, 6, 4),
    ].iter() {
        let test_name = format!("tasks_{}_depth_{}_branch_{}", task_count, depth, branching);
        
        group.bench_function(&test_name, |b| {
            let tasks: Vec<Task> = (0..*task_count)
                .map(|i| {
                    let x = (i as f64 % 100.0) * 50.0;
                    let y = (i as f64 / 100.0) * 50.0;
                    create_task_with_position(x, y)
                })
                .collect();
            
            b.iter(|| {
                let mut graph = DependencyGraph::new();
                for task in &tasks {
                    graph.add_task(task.id);
                }
                
                let dependencies = create_nested_dependencies(&tasks, *depth, *branching);
                for dep in &dependencies {
                    let _ = graph.add_dependency(dep);
                }
                
                let _ = graph.topological_sort();
                
                let estimates: HashMap<Uuid, f32> = tasks
                    .iter()
                    .map(|t| (t.id, t.estimated_hours.unwrap_or(1.0)))
                    .collect();
                let _ = graph.get_critical_path(&estimates);
                
                black_box(graph)
            });
        });
    }
    group.finish();
}

fn bench_dependency_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_operations");
    group.sample_size(10);
    
    let task_counts = [100, 500, 1000, 2500];
    
    for count in task_counts.iter() {
        let tasks: Vec<Task> = (0..*count)
            .map(|i| {
                let x = (i as f64 % 100.0) * 50.0;
                let y = (i as f64 / 100.0) * 50.0;
                create_task_with_position(x, y)
            })
            .collect();
        
        let mut graph = DependencyGraph::new();
        for task in &tasks {
            graph.add_task(task.id);
        }
        
        let dependencies = create_nested_dependencies(&tasks, 4, 3);
        for dep in &dependencies {
            let _ = graph.add_dependency(dep);
        }
        
        group.bench_function(
            &format!("can_start_task_{}", count),
            |b| {
                let completed_tasks: HashSet<Uuid> = tasks
                    .iter()
                    .take(count / 2)
                    .map(|t| t.id)
                    .collect();
                
                b.iter(|| {
                    for task in tasks.iter().skip(count / 2) {
                        black_box(graph.can_start_task(task.id, &completed_tasks));
                    }
                });
            },
        );
        
        group.bench_function(
            &format!("get_dependencies_{}", count),
            |b| {
                b.iter(|| {
                    for task in &tasks {
                        black_box(graph.get_dependencies(task.id));
                        black_box(graph.get_dependents(task.id));
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_kanban_zoom_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("kanban_zoom");
    group.sample_size(20);
    
    #[derive(Clone)]
    struct KanbanSimulation {
        tasks: Vec<Task>,
        viewport_width: f32,
        viewport_height: f32,
        zoom_level: f32,
        camera_x: f32,
        camera_y: f32,
    }
    
    impl KanbanSimulation {
        fn new(task_count: usize) -> Self {
            let tasks: Vec<Task> = (0..task_count)
                .map(|i| {
                    let x = (i as f64 % 100.0) * 50.0;
                    let y = (i as f64 / 100.0) * 50.0;
                    create_task_with_position(x, y)
                })
                .collect();
            
            Self {
                tasks,
                viewport_width: 1920.0,
                viewport_height: 1080.0,
                zoom_level: 1.0,
                camera_x: 0.0,
                camera_y: 0.0,
            }
        }
        
        fn zoom_in(&mut self) {
            self.zoom_level *= 1.2;
            self.zoom_level = self.zoom_level.min(5.0);
        }
        
        fn zoom_out(&mut self) {
            self.zoom_level /= 1.2;
            self.zoom_level = self.zoom_level.max(0.1);
        }
        
        fn pan(&mut self, dx: f32, dy: f32) {
            self.camera_x += dx / self.zoom_level;
            self.camera_y += dy / self.zoom_level;
        }
        
        fn get_visible_tasks(&self) -> Vec<&Task> {
            let left = self.camera_x;
            let right = self.camera_x + (self.viewport_width / self.zoom_level);
            let top = self.camera_y;
            let bottom = self.camera_y + (self.viewport_height / self.zoom_level);
            
            self.tasks
                .iter()
                .filter(|task| {
                    task.position.x >= left as f64 && 
                    task.position.x <= right as f64 &&
                    task.position.y >= top as f64 && 
                    task.position.y <= bottom as f64
                })
                .collect()
        }
        
        fn calculate_layout(&self) -> HashMap<TaskStatus, Vec<&Task>> {
            let mut layout = HashMap::new();
            for status in [
                TaskStatus::Todo,
                TaskStatus::InProgress,
                TaskStatus::Blocked,
                TaskStatus::Review,
                TaskStatus::Done,
            ] {
                layout.insert(status, Vec::new());
            }
            
            for task in self.get_visible_tasks() {
                layout.get_mut(&task.status).unwrap().push(task);
            }
            
            layout
        }
    }
    
    for count in [100, 500, 1000, 2500, 5000].iter() {
        group.bench_function(
            &format!("zoom_operations_{}", count),
            |b| {
                let mut sim = KanbanSimulation::new(*count);
                
                b.iter(|| {
                    for _ in 0..10 {
                        sim.zoom_in();
                        black_box(sim.calculate_layout());
                    }
                    
                    for _ in 0..20 {
                        sim.zoom_out();
                        black_box(sim.calculate_layout());
                    }
                    
                    for _ in 0..10 {
                        sim.zoom_in();
                        black_box(sim.calculate_layout());
                    }
                    
                    sim.zoom_level = 1.0;
                    black_box(sim.calculate_layout());
                });
            },
        );
        
        group.bench_function(
            &format!("pan_operations_{}", count),
            |b| {
                let mut sim = KanbanSimulation::new(*count);
                
                b.iter(|| {
                    for i in 0..50 {
                        let dx = (i as f32 * 10.0) % 500.0;
                        let dy = (i as f32 * 7.0) % 300.0;
                        sim.pan(dx, dy);
                        black_box(sim.get_visible_tasks());
                    }
                });
            },
        );
        
        group.bench_function(
            &format!("viewport_culling_{}", count),
            |b| {
                let sim = KanbanSimulation::new(*count);
                
                b.iter(|| {
                    for zoom in [0.1, 0.5, 1.0, 2.0, 5.0].iter() {
                        let mut test_sim = sim.clone();
                        test_sim.zoom_level = *zoom;
                        
                        for x in (0..5000).step_by(500) {
                            for y in (0..3000).step_by(300) {
                                test_sim.camera_x = x as f32;
                                test_sim.camera_y = y as f32;
                                black_box(test_sim.get_visible_tasks());
                            }
                        }
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_task_filtering_and_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_operations");
    group.sample_size(10);
    
    for count in [100, 500, 1000, 2500, 5000].iter() {
        let tasks: Vec<Task> = (0..*count)
            .map(|i| {
                let x = (i as f64 % 100.0) * 50.0;
                let y = (i as f64 / 100.0) * 50.0;
                create_task_with_position(x, y)
            })
            .collect();
        
        group.bench_function(
            &format!("filter_by_status_{}", count),
            |b| {
                b.iter(|| {
                    for status in [
                        TaskStatus::Todo,
                        TaskStatus::InProgress,
                        TaskStatus::Blocked,
                        TaskStatus::Review,
                        TaskStatus::Done,
                    ] {
                        let filtered: Vec<_> = tasks
                            .iter()
                            .filter(|t| t.status == status)
                            .collect();
                        black_box(filtered);
                    }
                });
            },
        );
        
        group.bench_function(
            &format!("filter_by_priority_{}", count),
            |b| {
                b.iter(|| {
                    for priority in [
                        Priority::Low,
                        Priority::Medium,
                        Priority::High,
                        Priority::Critical,
                    ] {
                        let filtered: Vec<_> = tasks
                            .iter()
                            .filter(|t| t.priority == priority)
                            .collect();
                        black_box(filtered);
                    }
                });
            },
        );
        
        group.bench_function(
            &format!("sort_by_multiple_criteria_{}", count),
            |b| {
                b.iter(|| {
                    let mut sorted = tasks.clone();
                    sorted.sort_by(|a, b| {
                        a.priority.cmp(&b.priority)
                            .then(a.due_date.cmp(&b.due_date))
                            .then(a.created_at.cmp(&b.created_at))
                    });
                    black_box(sorted);
                });
            },
        );
        
        group.bench_function(
            &format!("search_text_{}", count),
            |b| {
                let search_terms = vec!["test", "urgent", "bug", "feature", "todo"];
                
                b.iter(|| {
                    for term in &search_terms {
                        let found: Vec<_> = tasks
                            .iter()
                            .filter(|t| {
                                t.title.to_lowercase().contains(term) ||
                                t.description.to_lowercase().contains(term) ||
                                t.tags.iter().any(|tag| tag.to_lowercase().contains(term))
                            })
                            .collect();
                        black_box(found);
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_stress_test_complete_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_test");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("complete_system_10000_tasks", |b| {
        b.iter(|| {
            let task_count = 10000;
            let mut tasks = Vec::with_capacity(task_count);
            let mut graph = DependencyGraph::new();
            
            for i in 0..task_count {
                let x = (i as f64 % 200.0) * 50.0;
                let y = (i as f64 / 200.0) * 50.0;
                let task = create_task_with_position(x, y);
                graph.add_task(task.id);
                tasks.push(task);
            }
            
            let dependencies = create_nested_dependencies(&tasks, 7, 3);
            let mut successful_deps = 0;
            for dep in &dependencies {
                if graph.add_dependency(dep).is_ok() {
                    successful_deps += 1;
                }
            }
            
            if let Ok(sorted) = graph.topological_sort() {
                black_box(sorted.len());
            }
            
            let estimates: HashMap<Uuid, f32> = tasks
                .iter()
                .map(|t| (t.id, t.estimated_hours.unwrap_or(1.0)))
                .collect();
            let critical_path = graph.get_critical_path(&estimates);
            
            let mut completed = HashSet::new();
            let mut can_start = Vec::new();
            
            for task in &tasks {
                if graph.can_start_task(task.id, &completed) {
                    can_start.push(task.id);
                }
                
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) && rng.gen_bool(0.5) {
                    completed.insert(task.id);
                }
            }
            
            let mut status_groups: HashMap<TaskStatus, Vec<&Task>> = HashMap::new();
            for task in &tasks {
                status_groups.entry(task.status).or_insert_with(Vec::new).push(task);
            }
            
            let mut priority_groups: HashMap<Priority, Vec<&Task>> = HashMap::new();
            for task in &tasks {
                priority_groups.entry(task.priority).or_insert_with(Vec::new).push(task);
            }
            
            black_box((
                tasks.len(),
                successful_deps,
                critical_path.len(),
                can_start.len(),
                status_groups.len(),
                priority_groups.len(),
            ))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_create_thousands_of_cards,
    bench_nested_dependencies,
    bench_dependency_operations,
    bench_kanban_zoom_operations,
    bench_task_filtering_and_sorting,
    bench_stress_test_complete_system
);

criterion_main!(benches);