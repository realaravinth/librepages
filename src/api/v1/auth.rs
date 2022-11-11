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

use crate::ctx::api::v1::auth::{Login, Register};
use actix_identity::Identity;
use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};

use super::RedirectQuery;
use crate::errors::*;
use crate::AppCtx;

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
    cfg.service(login);
    cfg.service(signout);
}
#[actix_web_codegen_const_routes::post(path = "crate::V1_API_ROUTES.auth.register")]
#[tracing::instrument(name = "Register new user", skip(ctx, payload))]
async fn register(payload: web::Json<Register>, ctx: AppCtx) -> ServiceResult<impl Responder> {
    ctx.register(&payload).await?;
    Ok(HttpResponse::Ok())
}

#[actix_web_codegen_const_routes::post(path = "crate::V1_API_ROUTES.auth.login")]
#[tracing::instrument(name = "Login", skip(ctx, payload, id, query))]
async fn login(
    id: Identity,
    payload: web::Json<Login>,
    query: web::Query<RedirectQuery>,
    ctx: AppCtx,
) -> ServiceResult<impl Responder> {
    let payload = payload.into_inner();
    let username = ctx.login(&payload).await?;
    id.remember(username);
    let query = query.into_inner();
    if let Some(redirect_to) = query.redirect_to {
        Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, redirect_to))
            .finish())
    } else {
        Ok(HttpResponse::Ok().into())
    }
}

#[actix_web_codegen_const_routes::get(
    path = "crate::V1_API_ROUTES.auth.logout",
    wrap = "super::get_auth_middleware()"
)]
#[tracing::instrument(name = "Sign out", skip(id))]
async fn signout(id: Identity) -> impl Responder {
    use actix_auth_middleware::GetLoginRoute;

    if id.identity().is_some() {
        id.forget();
    }
    HttpResponse::Found()
        .append_header((header::LOCATION, crate::V1_API_ROUTES.get_login_route(None)))
        .finish()
}
