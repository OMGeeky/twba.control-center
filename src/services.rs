use crate::DatabaseConnection;
use crate::ResponderError;
use rocket::form::Form;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use twba_common::prelude::twba_local_db::prelude::Services;
use twba_common::prelude::twba_local_db::prelude::ServicesModel;
use twba_common::prelude::twba_local_db::prelude::Tasks;
use twba_common::prelude::twba_local_db::prelude::TasksModel;
use twba_common::prelude::twba_local_db::re_exports::sea_orm::ActiveModelTrait;
use twba_common::prelude::twba_local_db::re_exports::sea_orm::ActiveValue;
use twba_common::prelude::twba_local_db::re_exports::sea_orm::EntityTrait;
use twba_common::prelude::twba_local_db::re_exports::sea_orm::IntoActiveModel;

async fn get_services(db: &DatabaseConnection) -> Result<Vec<Service>, ResponderError> {
    let mut list = vec![];

    let services = Services::find().all(db).await?;
    let tasks = Tasks::find().all(db).await?;
    for service in services {
        let service_tasks = tasks
            .iter()
            .filter(|x| x.service_id == service.id)
            .map(|x| Task {
                description: x.description.clone().unwrap_or_default(),
                id: x.id,
                max_progress: x.max_progress,
                progress: x.progress,
                service_id: x.service_id,
            })
            .collect();
        list.push(Service {
            id: service.id,
            name: service.name,
            tasks: service_tasks,
            last_update: service.last_update.unwrap_or_default(),
        });
    }
    Ok(list)
}
#[derive(serde::Serialize)]
pub struct Service {
    id: i32,
    name: String,
    tasks: Vec<Task>,
    last_update: String,
}

#[derive(serde::Serialize, FromForm)]
pub struct IdValueForm<T> {
    id: i32,
    value: T,
}
#[derive(serde::Serialize)]
pub struct Task {
    id: i32,
    service_id: i32,
    description: String,
    progress: i32,
    max_progress: i32,
}

#[post("/edit-service", data = "<data>")]
pub async fn service_edit(
    db: &State<DatabaseConnection>,
    data: Form<IdValueForm<String>>,
) -> Result<(), ResponderError> {
    let data = data.into_inner();
    let model = get_service_by_id(data.id, db.inner()).await?;
    let mut model = model.into_active_model();
    let name_value: String = data.value;
    model.name = ActiveValue::Set(name_value);
    model.save(db.inner()).await?;
    Ok(())
}
#[post("/edit-task", data = "<data>")]
pub async fn task_edit(
    db: &State<DatabaseConnection>,
    data: Form<IdValueForm<String>>,
) -> Result<(), ResponderError> {
    let data = data.into_inner();
    let model = get_task_by_id(data.id, db.inner()).await?;
    let mut model = model.into_active_model();
    let description: String = data.value;
    model.description = ActiveValue::Set(Some(description));
    model.save(db.inner()).await?;
    Ok(())
}

#[get("/")]
pub(super) async fn service_index(
    db: &State<DatabaseConnection>,
) -> Result<Template, ResponderError> {
    let services = get_services(db.inner()).await?;

    Ok(Template::render(
        "services-overview",
        context! {services:services},
    ))
}

#[post("/add")]
pub async fn add(db: &State<DatabaseConnection>) -> Result<(), ResponderError> {
    let s = twba_common::prelude::twba_local_db::entities::services::ActiveModel {
        id: ActiveValue::NotSet,
        name: ActiveValue::Set("Test1".to_string()),
        last_update: ActiveValue::NotSet,
    };
    Services::insert(s).exec(db.inner()).await?;
    Ok(())
}
#[post("/increment-progress/<task>")]
pub async fn increment_task_progress(
    task: i32,
    db: &State<DatabaseConnection>,
) -> Result<(), ResponderError> {
    let db = db.inner();
    let task = get_task_by_id(task, db).await?;
    let progress = task.progress;
    let mut task = task.into_active_model();
    task.progress = ActiveValue::Set(progress + 1);
    task.save(db).await?;
    Ok(())
}

async fn get_task_by_id(task: i32, db: &DatabaseConnection) -> Result<TasksModel, ResponderError> {
    let task = Tasks::find_by_id(task)
        .one(db)
        .await?
        .ok_or(ResponderError::DbEntityNotFound {
            table: "Tasks",
            key: format!("{task}"),
        })?;
    Ok(task)
}

#[post("/<service>/increment-progress/<task>")]
pub async fn increment_progress(
    service: i32,
    task: i32,
    db_state: &State<DatabaseConnection>,
) -> Result<(), ResponderError> {
    let db = db_state.inner();

    let service = get_service_by_id(service, db).await?;

    let datetime = chrono::offset::Utc::now().to_rfc3339();

    let mut service = service.into_active_model();

    service.last_update = ActiveValue::Set(Some(datetime));
    increment_task_progress(task, db_state).await?;

    service.save(db).await?;
    Ok(())
}

async fn get_service_by_id(
    service: i32,
    db: &DatabaseConnection,
) -> Result<ServicesModel, ResponderError> {
    let service =
        Services::find_by_id(service)
            .one(db)
            .await?
            .ok_or(ResponderError::DbEntityNotFound {
                table: "Services",
                key: format!("{service}"),
            })?;
    Ok(service)
}

#[get("/update_progress")]
pub async fn update_progress(db: &State<DatabaseConnection>) -> Result<Template, ResponderError> {
    let services = get_services(db.inner()).await?;
    Ok(Template::render("services", context! {services:services}))
}
