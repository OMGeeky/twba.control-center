use crate::services::init_services;
use derive_more::{FromStr, FromStrError};
#[macro_use]
extern crate rocket;

use rocket::request::FromParam;
use rocket::tokio::time::{sleep, Duration};
use rocket_dyn_templates::Template;
use std::sync::OnceLock;
use twba_common::init_tracing;
use twba_common::prelude::twba_local_db;
use twba_common::prelude::twba_local_db::re_exports::sea_orm;
use twba_common::prelude::twba_local_db::re_exports::sea_orm::DatabaseConnection;
use twba_common::prelude::Conf;

static CLIENT: OnceLock<DatabaseConnection> = OnceLock::new();
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
fn get_config<'a>() -> &'a Conf {
    CONF.get_or_init(twba_common::get_config)
}
async fn get_client<'a>() -> Result<&'a DatabaseConnection, MainError> {
    match CLIENT.get() {
        Some(client) => Ok(client),
        None => {
            CLIENT
                .set(get_new_client().await?)
                .expect("Failed to set client after failing to get client");
            Ok(CLIENT.get().expect("we just initialized the client"))
        }
    }
}
async fn get_new_client<'a>() -> Result<DatabaseConnection, MainError> {
    Ok(twba_local_db::open_database(Some(&get_config().db_url)).await?)
}
#[rocket::main]
async fn main() -> Result<(), MainError> {
    let _guard = init_tracing("twba_uploader");
    info!("Hello world!");
    let services = init_services();
    let _rocket = rocket::build()
        .manage(services)
        .mount("/", routes![index, delay,])
        .mount(
            "/services/",
            routes![
                services::service,
                services::service_info,
                services::update_progress,
                services::increment_progress,
            ],
        )
        .attach(Template::fairing())
        .launch()
        .await?;

    Ok(())
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
    DbClient,
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
