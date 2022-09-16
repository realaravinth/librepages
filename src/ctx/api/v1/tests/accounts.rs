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
use crate::api::v1::account::{Email, Username};
use crate::ctx::api::v1::account::ChangePasswordReqest;
use crate::ctx::api::v1::auth::Password;
use crate::ctx::api::v1::auth::Register;
use crate::ctx::ArcCtx;
use crate::errors::*;
use crate::*;

#[actix_rt::test]
async fn postgrest_account_works() {
    let (_dir, ctx) = crate::tests::get_ctx().await;
    uname_email_exists_works(ctx.clone()).await;
    email_udpate_password_validation_del_userworks(ctx.clone()).await;
    username_update_works(ctx).await;
}

async fn uname_email_exists_works(ctx: ArcCtx) {
    const NAME: &str = "testuserexistsfoo";
    const NAME2: &str = "testuserexists22";
    const NAME3: &str = "testuserexists32";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "accotestsuser22@a.com";
    const EMAIL2: &str = "accotestsuser222@a.com";
    const EMAIL3: &str = "accotestsuser322@a.com";

    let _ = ctx.db.delete_user(NAME).await;
    let _ = ctx.db.delete_user(PASSWORD).await;
    let _ = ctx.db.delete_user(NAME2).await;
    let _ = ctx.db.delete_user(NAME3).await;

    // check username exists for non existent account
    println!("{:?}", ctx.username_exists(NAME).await);
    assert!(!ctx.username_exists(NAME).await.unwrap().exists);
    // check username email for non existent account
    assert!(!ctx.email_exists(EMAIL).await.unwrap().exists);

    let mut register_payload = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: PASSWORD.into(),
        email: EMAIL.into(),
    };
    ctx.register(&register_payload).await.unwrap();
    register_payload.username = NAME2.into();
    register_payload.email = EMAIL2.into();
    ctx.register(&register_payload).await.unwrap();

    // check username exists
    assert!(ctx.username_exists(NAME).await.unwrap().exists);
    assert!(ctx.username_exists(NAME2).await.unwrap().exists);
    // check email exists
    assert!(ctx.email_exists(EMAIL).await.unwrap().exists);

    // update username
    ctx.update_username(NAME2, NAME3).await.unwrap();
    assert!(!ctx.username_exists(NAME2).await.unwrap().exists);
    assert!(ctx.username_exists(NAME3).await.unwrap().exists);

    assert!(matches!(
        ctx.update_username(NAME3, NAME).await.err(),
        Some(ServiceError::UsernameTaken)
    ));

    // update email
    assert_eq!(
        ctx.set_email(NAME, EMAIL2).await.err(),
        Some(ServiceError::EmailTaken)
    );
    ctx.set_email(NAME, EMAIL3).await.unwrap();

    // change password
    let mut change_password_req = ChangePasswordReqest {
        password: PASSWORD.into(),
        new_password: NAME.into(),
        confirm_new_password: PASSWORD.into(),
    };
    assert_eq!(
        ctx.change_password(NAME, &change_password_req).await.err(),
        Some(ServiceError::PasswordsDontMatch)
    );

    change_password_req.confirm_new_password = NAME.into();
    ctx.change_password(NAME, &change_password_req)
        .await
        .unwrap();
}

async fn email_udpate_password_validation_del_userworks(ctx: ArcCtx) {
    const NAME: &str = "testuser32sd2";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "testuser12232@a.com2";
    const NAME2: &str = "eupdauser22";
    const EMAIL2: &str = "eupdauser22@a.com";

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
    let email_update_resp = actix_web::test::call_service(
        &app,
        post_request!(&email_payload, crate::V1_API_ROUTES.account.update_email)
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
        crate::V1_API_ROUTES.account.update_email,
        &email_payload,
        ServiceError::EmailTaken,
    )
    .await;

    // wrong password while deleting account
    let mut payload = Password {
        password: NAME.into(),
    };
    ctx.bad_post_req_test(
        NAME,
        PASSWORD,
        V1_API_ROUTES.account.delete,
        &payload,
        ServiceError::WrongPassword,
    )
    .await;

    // delete account
    payload.password = PASSWORD.into();
    let delete_user_resp = actix_web::test::call_service(
        &app,
        post_request!(&payload, crate::V1_API_ROUTES.account.delete)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;

    assert_eq!(delete_user_resp.status(), StatusCode::OK);

    // try to delete an account that doesn't exist
    let account_not_found_resp = actix_web::test::call_service(
        &app,
        post_request!(&payload, crate::V1_API_ROUTES.account.delete)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(account_not_found_resp.status(), StatusCode::NOT_FOUND);
    let txt: ErrorToResponse = actix_web::test::read_body_json(account_not_found_resp).await;
    assert_eq!(txt.error, format!("{}", ServiceError::AccountNotFound));
}

async fn username_update_works(ctx: ArcCtx) {
    const NAME: &str = "testuse23423rupda";
    const EMAIL: &str = "testu23423serupda@sss.com";
    const EMAIL2: &str = "testu234serupda2@sss.com";
    const PASSWORD: &str = "longpassword2";
    const NAME2: &str = "terstusrt23423ds";
    const NAME_CHANGE: &str = "terstu234234srtdsxx";

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
    let username_update_resp = actix_web::test::call_service(
        &app,
        post_request!(
            &username_udpate,
            crate::V1_API_ROUTES.account.update_username
        )
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
        V1_API_ROUTES.account.update_username,
        &username_udpate,
        ServiceError::UsernameTaken,
    )
    .await;
}
