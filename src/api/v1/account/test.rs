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
use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::test;

use super::*;
use crate::api::v1::ROUTES;
use crate::ctx::api::v1::auth::Password;
use crate::ctx::Ctx;
use crate::*;

#[actix_rt::test]
async fn postgrest_account_works() {
    let (_, ctx) = crate::tests::get_ctx().await;
    uname_email_exists_works(ctx.clone()).await;
    email_udpate_password_validation_del_userworks(ctx.clone()).await;
    username_update_works(ctx.clone()).await;
    update_password_works(ctx.clone()).await;
}

async fn uname_email_exists_works(ctx: Arc<Ctx>) {
    const NAME: &str = "testuserexists";
    const PASSWORD: &str = "longpasswordasdfa2";
    const EMAIL: &str = "testuserexists2@a.com";

    let _ = ctx.delete_user(NAME, PASSWORD).await;

    let (_, signin_resp) = ctx.register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(ctx).await;

    let mut payload = AccountCheckPayload { val: NAME.into() };

    let user_exists_resp = test::call_service(
        &app,
        post_request!(&payload, ROUTES.account.username_exists)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(user_exists_resp.status(), StatusCode::OK);
    let mut resp: AccountCheckResp = test::read_body_json(user_exists_resp).await;
    assert!(resp.exists);

    payload.val = PASSWORD.into();

    let user_doesnt_exist = test::call_service(
        &app,
        post_request!(&payload, ROUTES.account.username_exists)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(user_doesnt_exist.status(), StatusCode::OK);
    resp = test::read_body_json(user_doesnt_exist).await;
    assert!(!resp.exists);

    let email_doesnt_exist = test::call_service(
        &app,
        post_request!(&payload, ROUTES.account.email_exists)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(email_doesnt_exist.status(), StatusCode::OK);
    resp = test::read_body_json(email_doesnt_exist).await;
    assert!(!resp.exists);

    payload.val = EMAIL.into();

    let email_exist = test::call_service(
        &app,
        post_request!(&payload, ROUTES.account.email_exists)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(email_exist.status(), StatusCode::OK);
    resp = test::read_body_json(email_exist).await;
    assert!(resp.exists);
}

async fn email_udpate_password_validation_del_userworks(ctx: Arc<Ctx>) {
    const NAME: &str = "testuser2";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "testuser1@a.com2";
    const NAME2: &str = "eupdauser";
    const EMAIL2: &str = "eupdauser@a.com";

    let _ = ctx.delete_user(NAME, PASSWORD).await;
    let _ = ctx.delete_user(NAME2, PASSWORD).await;

    let _ = ctx.register_and_signin(NAME2, EMAIL2, PASSWORD).await;
    let (_creds, signin_resp) = ctx.register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(ctx).await;

    // update email
    let mut email_payload = Email {
        email: EMAIL.into(),
    };
    let email_update_resp = test::call_service(
        &app,
        post_request!(&email_payload, ROUTES.account.update_email)
            //post_request!(&email_payload, EMAIL_UPDATE)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(email_update_resp.status(), StatusCode::OK);

    // check duplicate email while duplicate email
    email_payload.email = EMAIL2.into();
    ctx.bad_post_req_test(
        NAME,
        PASSWORD,
        ROUTES.account.update_email,
        &email_payload,
        ServiceError::EmailTaken,
    )
    .await;

    // wrong password while deleteing account
    let mut payload = Password {
        password: NAME.into(),
    };
    ctx.bad_post_req_test(
        NAME,
        PASSWORD,
        ROUTES.account.delete,
        &payload,
        ServiceError::WrongPassword,
    )
    .await;

    // delete account
    payload.password = PASSWORD.into();
    let delete_user_resp = test::call_service(
        &app,
        post_request!(&payload, ROUTES.account.delete)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;

    assert_eq!(delete_user_resp.status(), StatusCode::OK);

    // try to delete an account that doesn't exist
    let account_not_found_resp = test::call_service(
        &app,
        post_request!(&payload, ROUTES.account.delete)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(account_not_found_resp.status(), StatusCode::NOT_FOUND);
    let txt: ErrorToResponse = test::read_body_json(account_not_found_resp).await;
    assert_eq!(txt.error, format!("{}", ServiceError::AccountNotFound));
}

async fn username_update_works(ctx: Arc<Ctx>) {
    const NAME: &str = "testuserupda";
    const EMAIL: &str = "testuserupda@sss.com";
    const EMAIL2: &str = "testuserupda2@sss.com";
    const PASSWORD: &str = "longpassword2";
    const NAME2: &str = "terstusrtds";
    const NAME_CHANGE: &str = "terstusrtdsxx";

    let _ = futures::join!(
        ctx.delete_user(NAME, PASSWORD),
        ctx.delete_user(NAME2, PASSWORD),
        ctx.delete_user(NAME_CHANGE, PASSWORD)
    );

    let _ = ctx.register_and_signin(NAME2, EMAIL2, PASSWORD).await;
    let (_creds, signin_resp) = ctx.register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(ctx).await;

    // update username
    let mut username_udpate = Username {
        username: NAME_CHANGE.into(),
    };
    let username_update_resp = test::call_service(
        &app,
        post_request!(&username_udpate, ROUTES.account.update_username)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(username_update_resp.status(), StatusCode::OK);

    // check duplicate username with duplicate username
    username_udpate.username = NAME2.into();
    ctx.bad_post_req_test(
        NAME_CHANGE,
        PASSWORD,
        ROUTES.account.update_username,
        &username_udpate,
        ServiceError::UsernameTaken,
    )
    .await;
}

async fn update_password_works(ctx: Arc<Ctx>) {
    const NAME: &str = "updatepassuser";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "updatepassuser@a.com";

    let _ = ctx.delete_user(NAME, PASSWORD).await;

    let (_, signin_resp) = ctx.register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(ctx).await;

    let new_password = "newpassword";

    let update_password = ChangePasswordReqest {
        password: PASSWORD.into(),
        new_password: new_password.into(),
        confirm_new_password: PASSWORD.into(),
    };

    let res = ctx.change_password(NAME, &update_password).await;
    assert!(res.is_err());
    assert_eq!(res, Err(ServiceError::PasswordsDontMatch));

    let update_password = ChangePasswordReqest {
        password: PASSWORD.into(),
        new_password: new_password.into(),
        confirm_new_password: new_password.into(),
    };

    assert!(ctx.change_password(NAME, &update_password).await.is_ok());

    let update_password = ChangePasswordReqest {
        password: new_password.into(),
        new_password: new_password.into(),
        confirm_new_password: PASSWORD.into(),
    };

    ctx.bad_post_req_test(
        NAME,
        new_password,
        ROUTES.account.update_password,
        &update_password,
        ServiceError::PasswordsDontMatch,
    )
    .await;

    let update_password = ChangePasswordReqest {
        password: PASSWORD.into(),
        new_password: PASSWORD.into(),
        confirm_new_password: PASSWORD.into(),
    };

    ctx.bad_post_req_test(
        NAME,
        new_password,
        ROUTES.account.update_password,
        &update_password,
        ServiceError::WrongPassword,
    )
    .await;

    let update_password = ChangePasswordReqest {
        password: new_password.into(),
        new_password: PASSWORD.into(),
        confirm_new_password: PASSWORD.into(),
    };

    let update_password_resp = test::call_service(
        &app,
        post_request!(&update_password, ROUTES.account.update_password)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(update_password_resp.status(), StatusCode::OK);
}
