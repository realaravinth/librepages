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

#[actix_web_codegen_const_routes::post(path = "crate::V1_API_ROUTES.deploy.update")]
#[tracing::instrument(name = "Update webpages", skip(payload, ctx))]
async fn update(payload: web::Json<DeployEvent>, ctx: AppCtx) -> ServiceResult<impl Responder> {
    let payload = payload.into_inner();
    ctx.update_site(&payload.secret, Some(payload.branch))
        .await?;
    Ok(HttpResponse::Ok())
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DeploySecret {
    pub secret: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
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

#[actix_web_codegen_const_routes::post(path = "crate::V1_API_ROUTES.deploy.info")]
#[tracing::instrument(name = "Get webpage deploy info", skip(payload, ctx))]
async fn deploy_info(
    payload: web::Json<DeploySecret>,
    ctx: AppCtx,
) -> ServiceResult<impl Responder> {
    if let Ok(page) = ctx.db.get_site_from_secret(&payload.secret).await {
        let resp = DeployInfo::from_page(&Page::from_site(&ctx.settings, page))?;
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
        const NAME: &str = "dplyupdwrkuser";
        const PASSWORD: &str = "longpasswordasdfa2";
        const EMAIL: &str = "dplyupdwrkuser@a.com";

        let (_dir, ctx) = tests::get_ctx().await;
        let _ = ctx.delete_user(NAME, PASSWORD).await;
        let (_, _signin_resp) = ctx.register_and_signin(NAME, EMAIL, PASSWORD).await;
        let page = ctx.add_test_site(NAME.into()).await;
        let app = get_app!(ctx).await;

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
        const NAME: &str = "dplyinfwrkuser";
        const PASSWORD: &str = "longpasswordasdfa2";
        const EMAIL: &str = "dplyinfwrkuser@a.com";

        let (_dir, ctx) = tests::get_ctx().await;
        let _ = ctx.delete_user(NAME, PASSWORD).await;
        let (_, _signin_resp) = ctx.register_and_signin(NAME, EMAIL, PASSWORD).await;
        let page = ctx.add_test_site(NAME.into()).await;
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
