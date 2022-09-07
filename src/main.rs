/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::env;
use std::sync::Arc;

use actix_web::{
    error::InternalError, http::StatusCode, middleware as actix_middleware, web::Data as WebData,
    web::JsonConfig, App, HttpServer,
};
use log::info;

mod ctx;
mod deploy;
mod errors;
mod git;
mod meta;
mod page;
mod routes;
mod serve;
mod settings;
#[cfg(test)]
mod tests;

pub use routes::ROUTES as V1_API_ROUTES;
pub use settings::Settings;

pub const CACHE_AGE: u32 = 604800;

pub const GIT_COMMIT_HASH: &str = env!("GIT_HASH");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub type AppCtx = WebData<Arc<ctx::Ctx>>;

#[cfg(not(tarpaulin_include))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    {
        const LOG_VAR: &str = "RUST_LOG";
        if env::var(LOG_VAR).is_err() {
            env::set_var("RUST_LOG", "info");
        }
    }

    let settings = Settings::new().unwrap();
    let ctx = WebData::new(ctx::Ctx::new(settings.clone()));

    pretty_env_logger::init();

    info!(
        "{}: {}.\nFor more information, see: {}\nBuild info:\nVersion: {} commit: {}",
        PKG_NAME, PKG_DESCRIPTION, PKG_HOMEPAGE, VERSION, GIT_COMMIT_HASH
    );

    info!("Starting server on: http://{}", settings.server.get_ip());

    HttpServer::new(move || {
        App::new()
            .wrap(actix_middleware::Logger::default())
            .wrap(actix_middleware::Compress::default())
            .app_data(ctx.clone())
            .app_data(get_json_err())
            .wrap(
                actix_middleware::DefaultHeaders::new()
                    .add(("Permissions-Policy", "interest-cohort=()")),
            )
            .wrap(actix_middleware::NormalizePath::new(
                actix_middleware::TrailingSlash::Trim,
            ))
            .configure(services)
    })
    .workers(settings.server.workers.unwrap_or_else(num_cpus::get))
    .bind(settings.server.get_ip())
    .unwrap()
    .run()
    .await
}

#[cfg(not(tarpaulin_include))]
pub fn get_json_err() -> JsonConfig {
    JsonConfig::default().error_handler(|err, _| {
        //debug!("JSON deserialization error: {:?}", &err);
        InternalError::new(err, StatusCode::BAD_REQUEST).into()
    })
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    routes::services(cfg);
}
