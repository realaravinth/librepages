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
use std::sync::Arc;

//use crate::api::v1::auth::{Login, Register};
use crate::ctx::api::v1::auth::{Login, Register};
use crate::ctx::Ctx;
use crate::errors::*;

#[actix_rt::test]
async fn postgrest_auth_works() {
    let (_dir, ctx) = crate::tests::get_ctx().await;
    auth_works(ctx).await;
}

async fn auth_works(ctx: Arc<Ctx>) {
    const NAME: &str = "testuser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testuser1@a.com";

    let _ = ctx.delete_user(NAME, PASSWORD).await;

    // 1. Register with email == None
    let mut register_payload = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: PASSWORD.into(),
        email: EMAIL.into(),
    };

    // registration: passwords don't match
    register_payload.confirm_password = NAME.into();
    assert!(matches!(
        ctx.register(&register_payload).await.err(),
        Some(ServiceError::PasswordsDontMatch)
    ));

    register_payload.confirm_password = PASSWORD.into();

    ctx.register(&register_payload).await.unwrap();
    // check if duplicate username is allowed
    assert!(matches!(
        ctx.register(&register_payload).await.err(),
        Some(ServiceError::UsernameTaken)
    ));

    // check if duplicate email is allowed
    let name = format!("{}dupemail", NAME);
    register_payload.username = name;
    assert!(matches!(
        ctx.register(&register_payload).await.err(),
        Some(ServiceError::EmailTaken)
    ));

    // Sign in with email
    let mut creds = Login {
        login: EMAIL.into(),
        password: PASSWORD.into(),
    };
    ctx.login(&creds).await.unwrap();

    // signin with username
    creds.login = NAME.into();
    ctx.login(&creds).await.unwrap();

    // sigining in with non-existent username
    creds.login = "nonexistantuser".into();
    assert!(matches!(
        ctx.login(&creds).await.err(),
        Some(ServiceError::AccountNotFound)
    ));

    // sigining in with non-existent email
    creds.login = "nonexistantuser@example.com".into();
    assert!(matches!(
        ctx.login(&creds).await.err(),
        Some(ServiceError::AccountNotFound)
    ));

    // sign in with incorrect password
    creds.login = NAME.into();
    creds.password = NAME.into();
    assert!(matches!(
        ctx.login(&creds).await.err(),
        Some(ServiceError::WrongPassword)
    ));

    // delete user
    ctx.delete_user(NAME, PASSWORD).await.unwrap();
}
