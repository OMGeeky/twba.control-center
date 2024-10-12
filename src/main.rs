#[macro_use]
extern crate rocket;
use derive_more::FromStr;
use derive_more::FromStrError;
use rocket::request::FromParam;
use rocket::response::Responder;
use rocket::tokio::time::{sleep, Duration};
use rocket::State;
use rocket_dyn_templates::Template;
use std::sync::OnceLock;
use twba_common::init_tracing;
use twba_common::prelude::twba_local_db;
use twba_common::prelude::twba_local_db::re_exports::sea_orm;
use twba_common::prelude::twba_local_db::re_exports::sea_orm::DatabaseConnection;
use twba_common::prelude::Conf;

static CONF: OnceLock<Conf> = OnceLock::new();

mod services;

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Waited for {} seconds", seconds)
}
#[get("/")]
fn index() -> &'static str {
    trace!("Called Index");
    "Hello, world!"
}
#[post("/migrate")]
async fn migrate(db: &State<DatabaseConnection>) -> Result<(), ResponderError> {
    twba_local_db::migrate_db(db.inner()).await?;
    Ok(())
}
fn get_config<'a>() -> &'a Conf {
    CONF.get_or_init(twba_common::get_config)
}

async fn get_new_client<'a>() -> Result<DatabaseConnection, MainError> {
    Ok(twba_local_db::open_database(Some(&get_config().db_url)).await?)
}
#[rocket::main]
async fn main() -> Result<(), MainError> {
    let _guard = init_tracing("twba_uploader");
    info!("Hello world!");
    let db = get_new_client().await?;
    let _rocket = rocket::build()
        .manage(db)
        .mount("/", routes![index, delay, migrate])
        .mount(
            "/services/",
            routes![
                services::service_index,
                services::task_edit,
                services::service_edit,
                services::update_progress,
                services::increment_progress,
                services::increment_task_progress,
                services::add,
            ],
        )
        .attach(Template::fairing())
        .launch()
        .await?;

    Ok(())
}
#[derive(Debug, derive_more::Error, derive_more::Display, Responder)]
#[display("{e}")]
pub struct DbErr {
    _dummy: String,
    #[response(ignore)]
    e: sea_orm::DbErr,
}
impl From<sea_orm::DbErr> for DbErr {
    fn from(e: twba_common::prelude::twba_local_db::re_exports::sea_orm::DbErr) -> Self {
        let _dummy = "Some DB Error".to_string();
        Self { _dummy, e }
    }
}
impl From<sea_orm::DbErr> for ResponderError {
    fn from(e: twba_common::prelude::twba_local_db::re_exports::sea_orm::DbErr) -> Self {
        let e: DbErr = e.into();
        e.into()
    }
}
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From, Responder)]
pub enum ResponderError {
    #[response(status = 404)]
    Db(#[from] DbErr),
    #[display("Could not find entity '{table}' with key:'{key}'")]
    #[response(status = 404)]
    #[from(ignore)]
    DbEntityNotFound {
        table: &'static str,
        #[response(ignore)]
        key: String,
    },
}
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
pub enum MainError {
    Rocket(#[from] rocket::Error),
    Db(#[from] sea_orm::DbErr),
    #[from(ignore)]
    SetStatics {
        problem_static: Statics,
    },
    #[from(ignore)]
    MissingStatic {
        problem_static: Statics,
    },
    Other {
        reason: String,
    },
}
#[derive(Debug, derive_more::Display, derive_more::From, Clone, Copy)]
pub enum Statics {
    Config,
}

#[derive(
    Debug,
    derive_more::Display,
    derive_more::From,
    derive_more::FromStr,
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    FromFormField,
)]
enum AvailableServices {
    Uploader,
    Downloader,
    Splitter,
}
impl<'a> FromParam<'a> for AvailableServices {
    type Error = FromStrError;
    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        AvailableServices::from_str(param)
    }
}
