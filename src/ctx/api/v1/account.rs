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
//! Account management utility datastructures and methods
use serde::{Deserialize, Serialize};

pub use super::auth;
use crate::ctx::Ctx;
use crate::db;
use crate::errors::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Data structure used in `*_exists` methods
pub struct AccountCheckResp {
    /// set to true if the attribute in question exists
    pub exists: bool,
}

/// Data structure used to change password of a registered user
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChangePasswordReqest {
    /// current password
    pub password: String,
    /// new password
    pub new_password: String,
    /// new password confirmation
    pub confirm_new_password: String,
}

impl Ctx {
    /// check if email exists on database
    pub async fn email_exists(&self, email: &str) -> ServiceResult<AccountCheckResp> {
        let resp = AccountCheckResp {
            exists: self.db.email_exists(email).await?,
        };

        Ok(resp)
    }

    /// update email
    pub async fn set_email(&self, username: &str, new_email: &str) -> ServiceResult<()> {
        self.creds.email(new_email)?;

        let username = self.creds.username(username)?;

        let payload = db::UpdateEmail {
            username: &username,
            new_email,
        };
        self.db.update_email(&payload).await?;
        Ok(())
    }

    /// check if email exists in database
    pub async fn username_exists(&self, username: &str) -> ServiceResult<AccountCheckResp> {
        let processed_uname = self.creds.username(username)?;
        let resp = AccountCheckResp {
            exists: self.db.username_exists(&processed_uname).await?,
        };
        Ok(resp)
    }

    /// update username of a registered user
    pub async fn update_username(
        &self,
        current_username: &str,
        new_username: &str,
    ) -> ServiceResult<String> {
        let processed_uname = self.creds.username(new_username)?;

        self.db
            .update_username(current_username, &processed_uname)
            .await?;

        Ok(processed_uname)
    }

    // returns Ok(()) upon successful authentication
    async fn authenticate(&self, username: &str, password: &str) -> ServiceResult<()> {
        use argon2_creds::Config;
        let username = self.creds.username(username)?;
        let resp = self
            .db
            .get_password(&db::Login::Username(&username))
            .await?;
        if Config::verify(&resp.hash, password)? {
            Ok(())
        } else {
            Err(ServiceError::WrongPassword)
        }
    }

    /// delete user
    pub async fn delete_user(&self, username: &str, password: &str) -> ServiceResult<()> {
        let username = self.creds.username(username)?;
        self.authenticate(&username, password).await?;
        self.db.delete_user(&username).await?;
        Ok(())
    }

    /// change password
    pub async fn change_password(
        &self,

        username: &str,
        payload: &ChangePasswordReqest,
    ) -> ServiceResult<()> {
        if payload.new_password != payload.confirm_new_password {
            return Err(ServiceError::PasswordsDontMatch);
        }

        self.authenticate(username, &payload.password).await?;

        let hash = self.creds.password(&payload.new_password)?;

        let username = self.creds.username(username)?;
        let db_payload = db::NameHash { username, hash };

        self.db.update_password(&db_payload).await?;

        Ok(())
    }
}
