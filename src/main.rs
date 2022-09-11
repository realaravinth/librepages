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

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    error::InternalError, http::StatusCode, middleware as actix_middleware, web::Data as WebData,
    web::JsonConfig, App, HttpServer,
};
use clap::{Parser, Subcommand};
use log::info;

mod api;
mod ctx;
mod db;
mod deploy;
mod errors;
mod git;
mod meta;
mod page;
mod preview;
mod routes;
mod serve;
mod settings;
#[cfg(test)]
mod tests;
mod utils;

use ctx::Ctx;
pub use routes::ROUTES as V1_API_ROUTES;
pub use settings::Settings;

pub const CACHE_AGE: u32 = 604800;

pub const GIT_COMMIT_HASH: &str = env!("GIT_HASH");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub type AppCtx = WebData<ctx::ArcCtx>;

//#[cfg(not(tarpaulin_include))]
//#[actix_web::main]
//async fn main() -> std::io::Result<()> {
//    {
//        const LOG_VAR: &str = "RUST_LOG";
//        if env::var(LOG_VAR).is_err() {
//            env::set_var("RUST_LOG", "info");
//        }
//    }
//
//    let settings = Settings::new().unwrap();
//    let ctx = WebData::new(ctx::Ctx::new(settings.clone()));
//
//    pretty_env_logger::init();
//
//    info!(
//        "{}: {}.\nFor more information, see: {}\nBuild info:\nVersion: {} commit: {}",
//        PKG_NAME, PKG_DESCRIPTION, PKG_HOMEPAGE, VERSION, GIT_COMMIT_HASH
//    );
//
//
//}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// run database migrations
    Migrate,

    /// run server
    Serve,
}

#[actix_web::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();

    let cli = Cli::parse();

    info!(
        "{}: {}.\nFor more information, see: {}\nBuild info:\nVersion: {} commit: {}",
        PKG_NAME, PKG_DESCRIPTION, PKG_HOMEPAGE, VERSION, GIT_COMMIT_HASH
    );

    let settings = Settings::new().unwrap();
    let ctx = Ctx::new(settings.clone()).await;
    let ctx = actix_web::web::Data::new(ctx);

    match &cli.command {
        Commands::Migrate => ctx.db.migrate().await.unwrap(),
        Commands::Serve => serve(settings, ctx).await.unwrap(),
    }
    Ok(())
}

async fn serve(settings: Settings, ctx: AppCtx) -> std::io::Result<()> {
    let ip = settings.server.get_ip();

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

#[cfg(not(tarpaulin_include))]
pub fn get_identity_service(settings: &Settings) -> IdentityService<CookieIdentityPolicy> {
    let cookie_secret = &settings.server.cookie_secret;
    IdentityService::new(
        CookieIdentityPolicy::new(cookie_secret.as_bytes())
            .path("/")
            .name("Authorization")
            //TODO change cookie age
            .max_age_secs(216000)
            .domain(&settings.server.domain)
            .secure(false),
    )
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    routes::services(cfg);
}
