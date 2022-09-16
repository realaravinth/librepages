/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
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

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::ctx::api::v1::account::*;
use crate::ctx::api::v1::auth::Password;
use crate::errors::*;
use crate::AppCtx;

#[cfg(test)]
pub mod test;

pub use super::auth;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountCheckPayload {
    pub val: String,
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(username_exists);
    cfg.service(set_username);
    cfg.service(email_exists);
    cfg.service(set_email);
    cfg.service(delete_account);
    cfg.service(update_user_password);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Email {
    pub email: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Username {
    pub username: String,
}

/// update username
#[actix_web_codegen_const_routes::post(
    path = "crate::V1_API_ROUTES.account.update_username",
    wrap = "super::get_auth_middleware()"
)]
async fn set_username(
    id: Identity,
    payload: web::Json<Username>,
    ctx: AppCtx,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();

    let new_name = ctx.update_username(&username, &payload.username).await?;

    id.forget();
    id.remember(new_name);

    Ok(HttpResponse::Ok())
}

#[actix_web_codegen_const_routes::post(path = "crate::V1_API_ROUTES.account.username_exists")]
async fn username_exists(
    payload: web::Json<AccountCheckPayload>,
    ctx: AppCtx,
) -> ServiceResult<impl Responder> {
    Ok(HttpResponse::Ok().json(ctx.username_exists(&payload.val).await?))
}

#[actix_web_codegen_const_routes::post(path = "crate::V1_API_ROUTES.account.email_exists")]
pub async fn email_exists(
    payload: web::Json<AccountCheckPayload>,
    ctx: AppCtx,
) -> ServiceResult<impl Responder> {
    Ok(HttpResponse::Ok().json(ctx.email_exists(&payload.val).await?))
}

/// update email
#[actix_web_codegen_const_routes::post(
    path = "crate::V1_API_ROUTES.account.update_email",
    wrap = "super::get_auth_middleware()"
)]
async fn set_email(
    id: Identity,
    payload: web::Json<Email>,
    ctx: AppCtx,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    ctx.set_email(&username, &payload.email).await?;
    Ok(HttpResponse::Ok())
}

#[actix_web_codegen_const_routes::post(
    path = "crate::V1_API_ROUTES.account.delete",
    wrap = "super::get_auth_middleware()"
)]
async fn delete_account(
    id: Identity,
    payload: web::Json<Password>,
    ctx: AppCtx,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();

    ctx.delete_user(&username, &payload.password).await?;
    id.forget();
    Ok(HttpResponse::Ok())
}

#[actix_web_codegen_const_routes::post(
    path = "crate::V1_API_ROUTES.account.update_password",
    wrap = "super::get_auth_middleware()"
)]
async fn update_user_password(
    id: Identity,
    ctx: AppCtx,

    payload: web::Json<ChangePasswordReqest>,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let payload = payload.into_inner();
    ctx.change_password(&username, &payload).await?;

    Ok(HttpResponse::Ok())
}
