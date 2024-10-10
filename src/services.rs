use crate::AvailableServices;
use rocket::fs::{relative, FileServer};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(serde::Serialize)]
pub struct Service {
    name: String,
    id: String,
    tasks: Vec<Task>,
}

#[derive(serde::Serialize)]
pub struct Task {
    name: String,
    id: String,
    progress: Arc<AtomicUsize>, // Progress in percentage
}

#[get("/<service>/info")]
pub(super) fn service_info(service: AvailableServices) -> String {
    format!("Here is some info about the service: name: {service}")
}
pub(super) fn init_services() -> Vec<Service> {
    let task1_progress = Arc::new(AtomicUsize::new(60));
    let task2_progress = Arc::new(AtomicUsize::new(100));
    let task3_progress = Arc::new(AtomicUsize::new(20));
    let task4_progress = Arc::new(AtomicUsize::new(80));

    vec![
        Service {
            name: "Service A".to_string(),
            id: "s1".to_string(),
            tasks: vec![
                Task {
                    name: "Task 1".to_string(),
                    id: "t1".to_string(),
                    progress: task1_progress,
                },
                Task {
                    name: "Task 2".to_string(),
                    id: "t2".to_string(),
                    progress: task2_progress,
                },
            ],
        },
        Service {
            name: "Service B".to_string(),
            id: "s2".to_string(),
            tasks: vec![
                Task {
                    name: "Task 3".to_string(),
                    id: "t3".to_string(),
                    progress: task3_progress,
                },
                Task {
                    name: "Task 4".to_string(),
                    id: "t4".to_string(),
                    progress: task4_progress,
                },
            ],
        },
    ]
}
#[get("/")]
pub(super) fn service(services: &State<Vec<Service>>) -> Template {
    let x = services.inner();
    let last_update = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Template::render(
        "services-overview",
        context! { services: x , last_update: last_update },
    )
}

#[post("/<service>/increment-progress/<task>")]
pub fn increment_progress(
    service: String,
    task: String,
    services: &State<Vec<Service>>,
) -> Result<(), String> {
    if let Some(service) = services.inner().iter().find(|x| x.id == service) {
        if let Some(task) = service.tasks.iter().find(|x| x.id == task) {
            task.progress.fetch_add(1, Ordering::AcqRel);
            Ok(())
        } else {
            Err("task with index not found".to_string())
        }
    } else {
        Err("service with index not found".to_string())
    }
}
#[get("/update_progress")]
pub fn update_progress(services: &State<Vec<Service>>) -> Template {
    Template::render("services", context! { services: services.inner() })
}
