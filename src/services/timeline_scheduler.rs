use crate::domain::{task::Task, resource::{Resource, ResourceAllocation}, dependency::{DependencyGraph, DependencyType}};
use chrono::{NaiveDate, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSchedule {
    pub task_id: Uuid,
    pub resource_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub allocated_hours: f32,
}

#[derive(Debug, Clone)]
pub struct TimelineSchedule {
    pub task_schedules: HashMap<Uuid, TaskSchedule>,
    pub resource_allocations: Vec<ResourceAllocation>,
    pub critical_path: Vec<Uuid>,
    pub warnings: Vec<String>,
}

impl TimelineSchedule {
    pub fn get_total_duration_days(&self) -> i64 {
        if self.task_schedules.is_empty() {
            return 0;
        }
        
        let min_date = self.task_schedules.values()
            .map(|s| s.start_date)
            .min()
            .unwrap();
            
        let max_date = self.task_schedules.values()
            .map(|s| s.end_date)
            .max()
            .unwrap();
            
        (max_date - min_date).num_days() + 1
    }
}

pub struct TimelineScheduler {
    // Resource availability tracking: resource_id -> date -> hours available
    resource_availability: HashMap<Uuid, HashMap<NaiveDate, f32>>,
}

impl Default for TimelineScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineScheduler {
    pub fn new() -> Self {
        Self {
            resource_availability: HashMap::new(),
        }
    }
    
    pub fn calculate_schedule(
        &mut self,
        tasks: &HashMap<Uuid, Task>,
        resources: &HashMap<Uuid, Resource>,
        dependency_graph: &DependencyGraph,
        start_date: NaiveDate,
    ) -> Result<TimelineSchedule, String> {
        let mut task_schedules = HashMap::new();
        let mut resource_allocations = Vec::new();
        let mut warnings = Vec::new();
        
        // Initialize resource availability
        self.resource_availability.clear();
        for resource_id in resources.keys() {
            self.resource_availability.insert(*resource_id, HashMap::new());
        }
        
        // Get tasks in topological order (respecting dependencies)
        let sorted_tasks = match dependency_graph.topological_sort() {
            Ok(sorted) => sorted,
            Err(_) => {
                // If no topological sort exists, just use all tasks
                tasks.keys().cloned().collect()
            }
        };
        
        // Filter to only tasks that exist in our task map
        let sorted_tasks: Vec<Uuid> = sorted_tasks.into_iter()
            .filter(|id| tasks.contains_key(id))
            .collect();
        
        // If no sorted tasks but we have tasks, add them
        let tasks_to_schedule = if sorted_tasks.is_empty() && !tasks.is_empty() {
            tasks.keys().cloned().collect()
        } else {
            sorted_tasks
        };
        
        // Schedule each task
        for task_id in tasks_to_schedule {
            let task = match tasks.get(&task_id) {
                Some(t) => t,
                None => continue,
            };
            
            // Check if task has an assigned resource
            let resource = task.assigned_resource_id
                .and_then(|id| resources.get(&id));
            
            if task.assigned_resource_id.is_none() {
                warnings.push(format!("Task '{}' is unassigned to any resource", task.title));
            }
            
            // Get estimated hours
            let estimated_hours = task.estimated_hours.unwrap_or(8.0);
            
            // Calculate earliest start date based on dependencies
            let earliest_start = self.calculate_earliest_start(
                &task_id,
                dependency_graph,
                &task_schedules,
                start_date,
            );
            
            // Schedule the task
            let schedule = if let Some(resource) = resource {
                self.schedule_task_with_resource(
                    task_id,
                    estimated_hours,
                    resource,
                    earliest_start,
                )?
            } else {
                // Schedule without resource constraints
                self.schedule_task_without_resource(
                    task_id,
                    estimated_hours,
                    earliest_start,
                )
            };
            
            // Create resource allocation if resource is assigned
            if let Some(resource_id) = task.assigned_resource_id {
                resource_allocations.push(ResourceAllocation::new(
                    resource_id,
                    task_id,
                    estimated_hours,
                    schedule.start_date,
                    schedule.end_date,
                ));
            }
            
            task_schedules.insert(task_id, schedule);
        }
        
        // Calculate critical path
        let task_estimates: HashMap<Uuid, f32> = tasks.iter()
            .map(|(id, task)| (*id, task.estimated_hours.unwrap_or(8.0)))
            .collect();
        let critical_path = dependency_graph.get_critical_path(&task_estimates);
        
        Ok(TimelineSchedule {
            task_schedules,
            resource_allocations,
            critical_path,
            warnings,
        })
    }
    
    fn calculate_earliest_start(
        &self,
        task_id: &Uuid,
        dependency_graph: &DependencyGraph,
        scheduled_tasks: &HashMap<Uuid, TaskSchedule>,
        default_start: NaiveDate,
    ) -> NaiveDate {
        let dependencies = dependency_graph.get_dependencies(*task_id);
        
        let mut earliest_start = default_start;
        
        for (dep_task_id, dep_type) in dependencies {
            if let Some(dep_schedule) = scheduled_tasks.get(&dep_task_id) {
                let required_start = match dep_type {
                    DependencyType::FinishToStart => {
                        // Start after dependency finishes
                        dep_schedule.end_date + chrono::Duration::days(1)
                    }
                    DependencyType::StartToStart => {
                        // Start when dependency starts
                        dep_schedule.start_date
                    }
                    DependencyType::FinishToFinish => {
                        // This is more complex - would need to work backwards from end date
                        dep_schedule.end_date
                    }
                    DependencyType::StartToFinish => {
                        // Rare case - finish when dependency starts
                        dep_schedule.start_date
                    }
                };
                
                if required_start > earliest_start {
                    earliest_start = required_start;
                }
            }
        }
        
        earliest_start
    }
    
    fn schedule_task_with_resource(
        &mut self,
        task_id: Uuid,
        estimated_hours: f32,
        resource: &Resource,
        earliest_start: NaiveDate,
    ) -> Result<TaskSchedule, String> {
        let mut current_date = earliest_start;
        let mut remaining_hours = estimated_hours;
        let mut start_date = None;
        
        // Get or initialize resource availability
        let resource_availability = self.resource_availability
            .get_mut(&resource.id)
            .ok_or("Resource not found in availability map")?;
        
        while remaining_hours > 0.0 {
            // Skip weekends
            if current_date.weekday().num_days_from_monday() >= 5 {
                current_date += chrono::Duration::days(1);
                continue;
            }
            
            // Get available hours for this resource on this date
            let daily_capacity = resource.get_availability_for_date(current_date);
            let already_allocated = *resource_availability.get(&current_date).unwrap_or(&0.0);
            let available = (daily_capacity - already_allocated).max(0.0);
            
            if available > 0.0 {
                if start_date.is_none() {
                    start_date = Some(current_date);
                }
                
                let hours_to_allocate = available.min(remaining_hours);
                remaining_hours -= hours_to_allocate;
                
                // Update resource availability
                resource_availability.insert(
                    current_date,
                    already_allocated + hours_to_allocate,
                );
            }
            
            if remaining_hours > 0.0 {
                current_date += chrono::Duration::days(1);
            }
        }
        
        let start_date = start_date.ok_or("Could not find available time for task")?;
        
        Ok(TaskSchedule {
            task_id,
            resource_id: Some(resource.id),
            start_date,
            end_date: current_date,
            allocated_hours: estimated_hours,
        })
    }
    
    fn schedule_task_without_resource(
        &self,
        task_id: Uuid,
        estimated_hours: f32,
        earliest_start: NaiveDate,
    ) -> TaskSchedule {
        // Assume 8 hours per day for unassigned tasks
        let days_needed = (estimated_hours / 8.0).ceil() as i64;
        let mut end_date = earliest_start;
        let mut days_added = 0;
        
        while days_added < days_needed {
            // Skip weekends
            if end_date.weekday().num_days_from_monday() < 5 {
                days_added += 1;
            }
            if days_added < days_needed {
                end_date += chrono::Duration::days(1);
            }
        }
        
        TaskSchedule {
            task_id,
            resource_id: None,
            start_date: earliest_start,
            end_date,
            allocated_hours: estimated_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timeline_schedule_duration() {
        let schedule = TimelineSchedule {
            task_schedules: HashMap::new(),
            resource_allocations: Vec::new(),
            critical_path: Vec::new(),
            warnings: Vec::new(),
        };
        
        assert_eq!(schedule.get_total_duration_days(), 0);
    }
}