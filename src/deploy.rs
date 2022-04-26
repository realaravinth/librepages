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
use tokio::sync::oneshot;

use crate::errors::*;
use crate::SETTINGS;

pub mod routes {
    pub struct Deploy {
        pub update: &'static str,
    }

    impl Deploy {
        pub const fn new() -> Self {
            Self {
                update: "/api/v1/update",
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeployEvent {
    pub secret: String,
    pub branch: String,
}

#[my_codegen::post(path = "crate::V1_API_ROUTES.deploy.update")]
async fn update(payload: web::Json<DeployEvent>) -> ServiceResult<impl Responder> {
    for page in SETTINGS.pages.iter() {
        if page.secret == payload.secret {
            let (tx, rx) = oneshot::channel();
            web::block(|| {
                tx.send(page.update()).unwrap();
            })
            .await
            .unwrap();
            rx.await.unwrap()?;
            return Ok(HttpResponse::Ok());
        }
    }

    Err(ServiceError::WebsiteNotFound)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(update);
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test, App};

    use crate::services;
    use crate::*;

    use super::*;

    #[actix_rt::test]
    async fn deploy_update_works() {
        let app = test::init_service(App::new().configure(services)).await;

        let page = SETTINGS.pages.get(0);
        let page = page.unwrap();

        let mut payload = DeployEvent {
            secret: page.secret.clone(),
            branch: page.branch.clone(),
        };

        let resp = test::call_service(
            &app,
            test::TestRequest::post()
                .uri(V1_API_ROUTES.deploy.update)
                .set_json(&payload)
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        payload.secret = page.branch.clone();

        let resp = test::call_service(
            &app,
            test::TestRequest::post()
                .uri(V1_API_ROUTES.deploy.update)
                .set_json(&payload)
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
