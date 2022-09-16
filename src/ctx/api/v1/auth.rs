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
//! Authentication helper methods and data structures
use serde::{Deserialize, Serialize};

use crate::ctx::Ctx;
use crate::db;
use crate::errors::*;

/// Register payload
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    /// username
    pub username: String,
    /// password
    pub password: String,
    /// password confirmation: `password` and `confirm_password` must match
    pub confirm_password: String,
    pub email: String,
}

/// Login payload
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Login {
    // login accepts both username and email under "username field"
    // TODO update all instances where login is used
    /// user identifier: either username or email
    /// an email is detected by checkinf for the existence of `@` character
    pub login: String,
    /// password
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// struct used to represent password
pub struct Password {
    /// password
    pub password: String,
}

impl Ctx {
    /// Log in method. Returns `Ok(())` when user is authenticated and errors when authentication
    /// fails
    pub async fn login(&self, payload: &Login) -> ServiceResult<String> {
        use argon2_creds::Config;

        let verify = |stored: &str, received: &str| {
            if Config::verify(stored, received)? {
                Ok(())
            } else {
                Err(ServiceError::WrongPassword)
            }
        };

        let creds = if payload.login.contains('@') {
            self.db
                .get_password(&db::Login::Email(&payload.login))
                .await?
        } else {
            self.db
                .get_password(&db::Login::Username(&payload.login))
                .await?
        };
        verify(&creds.hash, &payload.password)?;
        Ok(creds.username)
    }

    /// register new user
    pub async fn register(&self, payload: &Register) -> ServiceResult<()> {
        if !self.settings.allow_registration {
            return Err(ServiceError::ClosedForRegistration);
        }

        if payload.password != payload.confirm_password {
            return Err(ServiceError::PasswordsDontMatch);
        }
        let username = self.creds.username(&payload.username)?;
        let hash = self.creds.password(&payload.password)?;

        self.creds.email(&payload.email)?;

        let db_payload = db::Register {
            username: &username,
            hash: &hash,
            email: &payload.email,
        };

        self.db.register(&db_payload).await
    }
}
