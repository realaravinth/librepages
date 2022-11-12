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
//! Represents all the ways a trait can fail using this crate
use std::convert::From;
use std::io::Error as FSErrorInner;
use std::sync::Arc;

use actix_web::{
    error::ResponseError,
    http::{header, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use argon2_creds::errors::CredsError;
use config::ConfigError as ConfigErrorInner;
use derive_more::{Display, Error};
use git2::Error as GitError;
use serde::{Deserialize, Serialize};
use url::ParseError;

use crate::page::Page;

#[derive(Debug, Display, Error)]
pub struct FSError(#[display(fmt = "File System Error {}", _0)] pub FSErrorInner);

#[derive(Debug, Display, Error)]
pub struct ConfigError(#[display(fmt = "Configuration Error {}", _0)] pub ConfigErrorInner);

#[cfg(not(tarpaulin_include))]
impl PartialEq for FSError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

#[cfg(not(tarpaulin_include))]
impl PartialEq for ConfigError {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string().trim() == other.0.to_string().trim()
    }
}

#[cfg(not(tarpaulin_include))]
impl From<FSErrorInner> for ServiceError {
    fn from(e: FSErrorInner) -> Self {
        Self::FSError(FSError(e))
    }
}

#[cfg(not(tarpaulin_include))]
impl From<ConfigErrorInner> for ServiceError {
    fn from(e: ConfigErrorInner) -> Self {
        Self::ConfigError(ConfigError(e))
    }
}

#[derive(Debug, Display, PartialEq, Error)]
#[cfg(not(tarpaulin_include))]
/// Error data structure grouping various error subtypes
pub enum ServiceError {
    /// All non-specific errors are grouped under this category
    #[display(fmt = "internal server error")]
    InternalServerError,

    #[display(fmt = "The value you entered for URL is not a URL")] //405j
    /// The value you entered for url is not url"
    NotAUrl,
    #[display(fmt = "URL too long, maximum length can't be greater then 2048 characters")] //405
    /// URL too long, maximum length can't be greater then 2048 characters
    URLTooLong,

    #[display(fmt = "Website not found")]
    /// website not found
    WebsiteNotFound,

    #[display(fmt = "File not found")]
    /// File not found
    FileNotFound,

    /// when the a path configured for a page is already taken
    #[display(
        fmt = "Path already used for another website. lhs: {:?} rhs: {:?}",
        _0,
        _1
    )]
    PathTaken(Arc<Page>, Arc<Page>),

    /// when the a Secret configured for a page is already taken
    #[display(
        fmt = "Secret already used for another website. lhs: {:?} rhs: {:?}",
        _0,
        _1
    )]
    SecretTaken(Arc<Page>, Arc<Page>),

    /// when the a Repository URL configured for a page is already taken
    #[display(
        fmt = "Repository URL already configured for another website deployment. lhs: {:?} rhs: {:?}",
        _0,
        _1
    )]
    DuplicateRepositoryURL(Arc<Page>, Arc<Page>),

    #[display(fmt = "File System Error {}", _0)]
    FSError(FSError),

    #[display(fmt = "Unauthorized {}", _0)]
    UnauthorizedOperation(#[error(not(source))] String),

    #[display(fmt = "Bad request: {}", _0)]
    BadRequest(#[error(not(source))] String),

    #[display(fmt = "Configuration Error {}", _0)]
    ConfigError(ConfigError),

    #[display(fmt = "Git Error {}", _0)]
    GitError(GitError),

    #[display(fmt = "Branch {} not found", _0)]
    BranchNotFound(#[error(not(source))] String),

    /// Username is taken
    #[display(fmt = "Username is taken")]
    UsernameTaken,
    /// Email is taken
    #[display(fmt = "Email is taken")]
    EmailTaken,
    /// Account not found
    #[display(fmt = "Account not found")]
    AccountNotFound,

    #[display(
        fmt = "This server is is closed for registration. Contact admin if this is unexpecter"
    )]
    /// registration failure, server is is closed for registration
    ClosedForRegistration,

    #[display(fmt = "The value you entered for email is not an email")] //405j
    /// The value you entered for email is not an email"
    NotAnEmail,

    #[display(fmt = "Wrong password")]
    /// wrong password
    WrongPassword,

    /// when the value passed contains profanity
    #[display(fmt = "Can't allow profanity in usernames")]
    ProfanityError,
    /// when the value passed contains blacklisted words
    /// see [blacklist](https://github.com/shuttlecraft/The-Big-Username-Blacklist)
    #[display(fmt = "Username contains blacklisted words")]
    BlacklistError,
    /// when the value passed contains characters not present
    /// in [UsernameCaseMapped](https://tools.ietf.org/html/rfc8265#page-7)
    /// profile
    #[display(fmt = "username_case_mapped violation")]
    UsernameCaseMappedError,

    #[display(fmt = "Passsword too short")]
    /// password too short
    PasswordTooShort,
    #[display(fmt = "password too long")]
    /// password too long
    PasswordTooLong,
    #[display(fmt = "Passwords don't match")]
    /// passwords don't match
    PasswordsDontMatch,
}

impl From<ParseError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(_: ParseError) -> ServiceError {
        ServiceError::NotAUrl
    }
}

impl From<GitError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(e: GitError) -> ServiceError {
        ServiceError::GitError(e)
    }
}

/// Generic result data structure
#[cfg(not(tarpaulin_include))]
pub type ServiceResult<V> = std::result::Result<V, ServiceError>;

#[derive(Serialize, Deserialize, Debug)]
#[cfg(not(tarpaulin_include))]
pub struct ErrorToResponse {
    pub error: String,
}

#[cfg(not(tarpaulin_include))]
impl ResponseError for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .append_header((header::CONTENT_TYPE, "application/json; charset=UTF-8"))
            .body(
                serde_json::to_string(&ErrorToResponse {
                    error: self.to_string(),
                })
                .unwrap(),
            )
    }

    #[cfg(not(tarpaulin_include))]
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR, // INTERNAL SERVER ERROR
            ServiceError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR, // INTERNAL SERVER ERROR
            ServiceError::NotAUrl => StatusCode::BAD_REQUEST,                  //BADREQUEST,
            ServiceError::URLTooLong => StatusCode::BAD_REQUEST,               //BADREQUEST,
            ServiceError::WebsiteNotFound => StatusCode::NOT_FOUND,            //NOT FOUND,

            ServiceError::PathTaken(_, _) => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::DuplicateRepositoryURL(_, _) => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::SecretTaken(_, _) => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::FSError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            ServiceError::UnauthorizedOperation(_) => StatusCode::UNAUTHORIZED,
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::GitError(_) => StatusCode::BAD_REQUEST,
            ServiceError::BranchNotFound(_) => StatusCode::CONFLICT,

            ServiceError::EmailTaken => StatusCode::BAD_REQUEST,
            ServiceError::UsernameTaken => StatusCode::BAD_REQUEST,
            ServiceError::AccountNotFound => StatusCode::NOT_FOUND,
            ServiceError::FileNotFound => StatusCode::NOT_FOUND,

            ServiceError::ProfanityError => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::BlacklistError => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::UsernameCaseMappedError => StatusCode::BAD_REQUEST, //BADREQUEST,

            ServiceError::PasswordTooShort => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::PasswordTooLong => StatusCode::BAD_REQUEST,  //BADREQUEST,
            ServiceError::PasswordsDontMatch => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::ClosedForRegistration => StatusCode::FORBIDDEN, //FORBIDDEN,
            ServiceError::NotAnEmail => StatusCode::BAD_REQUEST,       //BADREQUEST,
            ServiceError::WrongPassword => StatusCode::UNAUTHORIZED,   //UNAUTHORIZED,
        }
    }
}

impl From<CredsError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(e: CredsError) -> ServiceError {
        match e {
            CredsError::UsernameCaseMappedError => ServiceError::UsernameCaseMappedError,
            CredsError::ProfainityError => ServiceError::ProfanityError,
            CredsError::BlacklistError => ServiceError::BlacklistError,
            CredsError::NotAnEmail => ServiceError::NotAnEmail,
            CredsError::Argon2Error(_) => ServiceError::InternalServerError,
            CredsError::PasswordTooLong => ServiceError::PasswordTooLong,
            CredsError::PasswordTooShort => ServiceError::PasswordTooShort,
        }
    }
}
