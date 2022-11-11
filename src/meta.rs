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
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::{AppCtx, GIT_COMMIT_HASH, VERSION};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildDetails<'a> {
    pub version: &'a str,
    pub git_commit_hash: &'a str,
    pub source_code: &'a str,
}

pub mod routes {
    pub struct Meta {
        pub build_details: &'static str,
        pub health: &'static str,
    }

    impl Meta {
        pub const fn new() -> Self {
            Self {
                build_details: "/api/v1/meta/build",
                health: "/api/v1/meta/health",
            }
        }
    }
}

/// emits build details of the binary
#[actix_web_codegen_const_routes::get(path = "crate::V1_API_ROUTES.meta.build_details")]
#[tracing::instrument(name = "Fetch Build Details", skip(ctx))]
async fn build_details(ctx: AppCtx) -> impl Responder {
    let build = BuildDetails {
        version: VERSION,
        git_commit_hash: GIT_COMMIT_HASH,
        source_code: &ctx.settings.source_code,
    };
    HttpResponse::Ok().json(build)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Health check return datatype
pub struct Health {
    db: bool,
}

/// checks all components of the system
#[actix_web_codegen_const_routes::get(path = "crate::V1_API_ROUTES.meta.health")]
#[tracing::instrument(name = "Fetch health", skip(ctx))]
async fn health(ctx: crate::AppCtx) -> impl Responder {
    let res = Health {
        db: ctx.db.ping().await,
    };

    HttpResponse::Ok().json(res)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(build_details);
    cfg.service(health);
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test};

    use crate::*;

    #[actix_rt::test]
    async fn build_details_works() {
        let (_dir, ctx) = tests::get_ctx().await;
        println!("[log] test configuration {:#?}", ctx.settings);
        let app = get_app!(ctx).await;

        let resp = get_request!(app, V1_API_ROUTES.meta.build_details);
        check_status!(resp, StatusCode::OK);
    }

    #[actix_rt::test]
    async fn health_works() {
        use actix_web::test;

        let (_dir, ctx) = tests::get_ctx().await;
        let app = get_app!(ctx).await;

        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(crate::V1_API_ROUTES.meta.health)
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        let health_resp: super::Health = test::read_body_json(resp).await;
        assert!(health_resp.db);
    }
}
