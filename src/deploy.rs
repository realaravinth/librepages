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
use crate::page::Page;
use crate::AppCtx;

pub mod routes {
    pub struct Deploy {
        pub update: &'static str,
        pub info: &'static str,
    }

    impl Deploy {
        pub const fn new() -> Self {
            Self {
                update: "/api/v1/update",
                info: "/api/v1/info",
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeployEvent {
    pub secret: String,
    pub branch: String,
}

fn find_page<'a>(secret: &str, ctx: &'a AppCtx) -> Option<&'a Page> {
    for page in ctx.settings.pages.iter() {
        if page.secret == secret {
            return Some(page);
        }
    }
    None
}

#[my_codegen::post(path = "crate::V1_API_ROUTES.deploy.update")]
async fn update(payload: web::Json<DeployEvent>, ctx: AppCtx) -> ServiceResult<impl Responder> {
    if let Some(page) = find_page(&payload.secret, &ctx) {
        let (tx, rx) = oneshot::channel();
        let page = page.clone();
        web::block(move || {
            tx.send(page.update()).unwrap();
        })
        .await
        .unwrap();
        rx.await.unwrap()?;
        Ok(HttpResponse::Ok())
    } else {
        Err(ServiceError::WebsiteNotFound)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DeploySecret {
    pub secret: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DeployInfo {
    pub head: String,
    pub remote: String,
    pub commit: String,
}

impl DeployInfo {
    pub fn from_page(page: &Page) -> ServiceResult<Self> {
        let repo = page.open_repo()?;
        let head = page.get_deploy_branch(&repo)?;
        let commit = Page::get_deploy_commit(&repo)?.to_string();
        let remote = Page::get_deploy_remote(&repo)?;
        let remote = remote.url().unwrap().to_owned();

        Ok(Self {
            head,
            remote,
            commit,
        })
    }
}

#[my_codegen::post(path = "crate::V1_API_ROUTES.deploy.info")]
async fn deploy_info(
    payload: web::Json<DeploySecret>,
    ctx: AppCtx,
) -> ServiceResult<impl Responder> {
    if let Some(page) = find_page(&payload.secret, &ctx) {
        let resp = DeployInfo::from_page(page)?;
        Ok(HttpResponse::Ok().json(resp))
    } else {
        Err(ServiceError::WebsiteNotFound)
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(update);
    cfg.service(deploy_info);
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test};

    use crate::tests;
    use crate::*;

    use super::*;

    #[actix_rt::test]
    async fn deploy_update_works() {
        let (_dir, ctx) = tests::get_data().await;
        println!("[log] test configuration {:#?}", ctx.settings);
        let app = get_app!(ctx).await;
        let page = ctx.settings.pages.get(0);
        let page = page.unwrap();

        let mut payload = DeployEvent {
            secret: page.secret.clone(),
            branch: page.branch.clone(),
        };

        let resp = test::call_service(
            &app,
            post_request!(&payload, V1_API_ROUTES.deploy.update).to_request(),
        )
        .await;
        check_status!(resp, StatusCode::OK);

        payload.secret = page.branch.clone();

        let resp = test::call_service(
            &app,
            post_request!(&payload, V1_API_ROUTES.deploy.update).to_request(),
        )
        .await;
        check_status!(resp, StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn deploy_info_works() {
        let (_dir, ctx) = tests::get_data().await;
        println!("[log] test configuration {:#?}", ctx.settings);
        let page = ctx.settings.pages.get(0);
        let page = page.unwrap();
        let app = get_app!(ctx).await;

        let mut payload = DeploySecret {
            secret: page.secret.clone(),
        };

        let resp = test::call_service(
            &app,
            post_request!(&payload, V1_API_ROUTES.deploy.info).to_request(),
        )
        .await;

        check_status!(resp, StatusCode::OK);

        let response: DeployInfo = actix_web::test::read_body_json(resp).await;
        assert_eq!(response.head, page.branch);
        assert_eq!(response.remote, page.repo);

        payload.secret = page.branch.clone();

        let resp = test::call_service(
            &app,
            post_request!(&payload, V1_API_ROUTES.deploy.info).to_request(),
        )
        .await;
        check_status!(resp, StatusCode::NOT_FOUND);
    }
}
