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
use std::str::FromStr;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::time::OffsetDateTime;
use sqlx::ConnectOptions;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::errors::*;

/// Connect to databse
pub enum ConnectionOptions {
    /// fresh connection
    Fresh(Fresh),
    /// existing connection
    Existing(Conn),
}

/// Use an existing database pool
pub struct Conn(pub PgPool);

pub struct Fresh {
    pub pool_options: PgPoolOptions,
    pub disable_logging: bool,
    pub url: String,
}

impl ConnectionOptions {
    async fn connect(self) -> ServiceResult<Database> {
        let pool = match self {
            Self::Fresh(fresh) => {
                let mut connect_options =
                    sqlx::postgres::PgConnectOptions::from_str(&fresh.url).unwrap();
                if fresh.disable_logging {
                    connect_options.disable_statement_logging();
                }
                sqlx::postgres::PgConnectOptions::from_str(&fresh.url)
                    .unwrap()
                    .disable_statement_logging();
                fresh
                    .pool_options
                    .connect_with(connect_options)
                    .await
                    .unwrap()
                //.map_err(|e| ServiceError::ServiceError(Box::new(e)))?
            }

            Self::Existing(conn) => conn.0,
        };
        Ok(Database { pool })
    }
}

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn migrate(&self) -> ServiceResult<()> {
        sqlx::migrate!("./migrations/")
            .run(&self.pool)
            .await
            .unwrap();
        //.map_err(|e| ServiceError::ServiceError(Box::new(e)))?;
        self.create_event_type().await?;
        Ok(())
    }

    pub async fn ping(&self) -> bool {
        use sqlx::Connection;

        if let Ok(mut con) = self.pool.acquire().await {
            con.ping().await.is_ok()
        } else {
            false
        }
    }

    /// register a new user
    pub async fn register(&self, p: &Register<'_>) -> ServiceResult<()> {
        sqlx::query!(
            "INSERT INTO librepages_users
            (name , password, email) VALUES ($1, $2, $3)",
            &p.username,
            &p.hash,
            &p.email,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    /// delete a user
    pub async fn delete_user(&self, username: &str) -> ServiceResult<()> {
        sqlx::query!("DELETE FROM librepages_users WHERE name = ($1)", username)
            .execute(&self.pool)
            .await
            .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;
        Ok(())
    }

    /// check if username exists
    pub async fn username_exists(&self, username: &str) -> ServiceResult<bool> {
        let res = sqlx::query!(
            "SELECT EXISTS (SELECT 1 from librepages_users WHERE name = $1)",
            username,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_register_err)?;

        let mut resp = false;
        if let Some(x) = res.exists {
            resp = x;
        }

        Ok(resp)
    }

    /// get user email
    pub async fn get_email(&self, username: &str) -> ServiceResult<String> {
        struct Email {
            email: String,
        }

        let res = sqlx::query_as!(
            Email,
            "SELECT email FROM librepages_users WHERE name = $1",
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;
        Ok(res.email)
    }

    /// check if email exists
    pub async fn email_exists(&self, email: &str) -> ServiceResult<bool> {
        let res = sqlx::query!(
            "SELECT EXISTS (SELECT 1 from librepages_users WHERE email = $1)",
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_register_err)?;

        let mut resp = false;
        if let Some(x) = res.exists {
            resp = x;
        }

        Ok(resp)
    }

    /// update a user's email
    pub async fn update_email(&self, p: &UpdateEmail<'_>) -> ServiceResult<()> {
        sqlx::query!(
            "UPDATE librepages_users set email = $1
            WHERE name = $2",
            &p.new_email,
            &p.username,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;

        Ok(())
    }

    /// get a user's password
    pub async fn get_password(&self, l: &Login<'_>) -> ServiceResult<NameHash> {
        struct Password {
            name: String,
            password: String,
        }

        let rec = match l {
            Login::Username(u) => sqlx::query_as!(
                Password,
                r#"SELECT name, password  FROM librepages_users WHERE name = ($1)"#,
                u,
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?,
            Login::Email(e) => sqlx::query_as!(
                Password,
                r#"SELECT name, password  FROM librepages_users WHERE email = ($1)"#,
                e,
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?,
        };

        let res = NameHash {
            hash: rec.password,
            username: rec.name,
        };

        Ok(res)
    }

    /// update user's password
    pub async fn update_password(&self, p: &NameHash) -> ServiceResult<()> {
        sqlx::query!(
            "UPDATE librepages_users set password = $1
            WHERE name = $2",
            &p.hash,
            &p.username,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;

        Ok(())
    }

    /// update username
    pub async fn update_username(&self, current: &str, new: &str) -> ServiceResult<()> {
        sqlx::query!(
            "UPDATE librepages_users set name = $1
            WHERE name = $2",
            new,
            current,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;

        Ok(())
    }

    pub async fn add_site(&self, msg: &Site) -> ServiceResult<()> {
        sqlx::query!(
            "
            INSERT INTO librepages_sites
                (site_secret, repo_url, branch, hostname, owned_by)
            VALUES ($1, $2, $3, $4, ( SELECT ID FROM librepages_users WHERE name = $5 ));
            ",
            msg.site_secret,
            msg.repo_url,
            msg.branch,
            msg.hostname,
            msg.owner,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;

        Ok(())
    }

    pub async fn get_site_from_secret(&self, site_secret: &str) -> ServiceResult<Site> {
        struct S {
            repo_url: String,
            branch: String,
            hostname: String,
            owned_by: i32,
        }

        let site = sqlx::query_as!(
            S,
            "SELECT repo_url, branch, hostname, owned_by
            FROM librepages_sites
            WHERE site_secret = $1
            ",
            site_secret,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::WebsiteNotFound))?;

        struct Owner {
            name: String,
        }
        let owner = sqlx::query_as!(
            Owner,
            "SELECT name FROM librepages_users WHERE ID = $1",
            site.owned_by
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::WebsiteNotFound))?;

        let site = Site {
            site_secret: site_secret.to_owned(),
            branch: site.branch,
            hostname: site.hostname,
            owner: owner.name,
            repo_url: site.repo_url,
        };

        Ok(site)
    }

    pub async fn get_site(&self, owner: &str, hostname: &str) -> ServiceResult<Site> {
        let site = sqlx::query_as!(
            InnerSite,
            "SELECT site_secret, repo_url, branch, hostname
            FROM librepages_sites
            WHERE owned_by = (SELECT ID FROM librepages_users WHERE name = $1 )
            AND hostname = $2;
            ",
            owner,
            hostname
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::WebsiteNotFound))?;

        let res = site.to_site(owner.into());

        Ok(res)
    }

    pub async fn list_all_sites(&self, owner: &str) -> ServiceResult<Vec<Site>> {
        let mut sites = sqlx::query_as!(
            InnerSite,
            "SELECT site_secret, repo_url, branch, hostname
            FROM librepages_sites
            WHERE owned_by = (SELECT ID FROM librepages_users WHERE name = $1 );
            ",
            owner,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;

        let res = sites.drain(0..).map(|s| s.to_site(owner.into())).collect();

        Ok(res)
    }

    pub async fn delete_site(&self, owner: &str, hostname: &str) -> ServiceResult<()> {
        sqlx::query!(
            "DELETE FROM librepages_sites
            WHERE hostname = ($1)
            AND owned_by = ( SELECT ID FROM librepages_users WHERE name = $2);
            ",
            hostname,
            owner
        )
        .execute(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::WebsiteNotFound))?;
        Ok(())
    }

    /// check if hostname exists
    pub async fn hostname_exists(&self, hostname: &str) -> ServiceResult<bool> {
        let res = sqlx::query!(
            "SELECT EXISTS (SELECT 1 from librepages_sites WHERE hostname = $1)",
            hostname,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_register_err)?;

        let mut resp = false;
        if let Some(x) = res.exists {
            resp = x;
        }

        Ok(resp)
    }

    /// check if event type exists
    async fn event_type_exists(&self, event: &Event) -> ServiceResult<bool> {
        let res = sqlx::query!(
            "SELECT EXISTS (SELECT 1 from librepages_deploy_event_type WHERE name = $1)",
            event.name,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_register_err)?;

        let mut resp = false;
        if let Some(x) = res.exists {
            resp = x;
        }

        Ok(resp)
    }

    async fn create_event_type(&self) -> ServiceResult<()> {
        for e in &*EVENTS {
            if !self.event_type_exists(e).await? {
                sqlx::query!(
                    "INSERT INTO librepages_deploy_event_type
                    (name) VALUES ($1) ON CONFLICT (name) DO NOTHING;",
                    e.name
                )
                .execute(&self.pool)
                .await
                .map_err(map_register_err)?;
            }
        }
        Ok(())
    }

    pub async fn log_event(&self, hostname: &str, event: &Event) -> ServiceResult<Uuid> {
        let now = now_unix_time_stamp();
        let uuid = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO librepages_site_deploy_events
            (event_type, time, site, pub_id) VALUES (
                (SELECT iD from librepages_deploy_event_type WHERE name = $1),
                $2,
                (SELECT ID from librepages_sites WHERE hostname = $3),
                $4
            );
            ",
            event.name,
            &now,
            hostname,
            uuid,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(uuid)
    }

    pub async fn get_event(
        &self,
        hostname: &str,
        event_id: &Uuid,
    ) -> ServiceResult<LibrePagesEvent> {
        let event = sqlx::query_as!(
            InnerLibrepagesEvent,
            "SELECT
                    librepages_deploy_event_type.name,
                    librepages_site_deploy_events.time,
                    librepages_site_deploy_events.pub_id
                FROM
                    librepages_site_deploy_events
                INNER JOIN librepages_deploy_event_type ON
                    librepages_deploy_event_type.ID = librepages_site_deploy_events.event_type
                WHERE
                    librepages_site_deploy_events.site = (
                        SELECT ID FROM librepages_sites WHERE hostname = $1
                    )
                AND
                    librepages_site_deploy_events.pub_id = $2
                ",
            hostname,
            event_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;

        Ok(LibrePagesEvent {
            id: event.pub_id,
            time: event.time,
            event_type: Event::from_str(&event.name).unwrap(),
            site: hostname.to_owned(),
        })
    }

    pub async fn list_all_site_events(
        &self,
        hostname: &str,
    ) -> ServiceResult<Vec<LibrePagesEvent>> {
        let mut inner_events = sqlx::query_as!(
            InnerLibrepagesEvent,
            "SELECT
                    librepages_deploy_event_type.name,
                    librepages_site_deploy_events.time,
                    librepages_site_deploy_events.pub_id
                FROM
                    librepages_site_deploy_events
                INNER JOIN librepages_deploy_event_type ON
                    librepages_deploy_event_type.ID = librepages_site_deploy_events.event_type
                WHERE
                    librepages_site_deploy_events.site = (
                        SELECT ID FROM librepages_sites WHERE hostname = $1
                    );
                ",
            hostname,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_row_not_found_err(e, ServiceError::AccountNotFound))?;

        let mut events = Vec::with_capacity(inner_events.len());

        for e in inner_events.drain(0..) {
            events.push(LibrePagesEvent {
                id: e.pub_id,
                time: e.time,
                event_type: Event::from_str(&e.name).unwrap(),
                site: hostname.to_owned(),
            })
        }
        Ok(events)
    }
}
struct InnerSite {
    site_secret: String,
    repo_url: String,
    branch: String,
    hostname: String,
}

impl InnerSite {
    fn to_site(self, owner: String) -> Site {
        Site {
            site_secret: self.site_secret,
            repo_url: self.repo_url,
            branch: self.branch,
            hostname: self.hostname,
            owner,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Data required to add a new site
pub struct Site {
    pub site_secret: String,
    pub repo_url: String,
    pub branch: String,
    pub hostname: String,
    pub owner: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Data required to register a new user
pub struct Register<'a> {
    /// username of new user
    pub username: &'a str,
    /// hashed password of new use
    pub hash: &'a str,
    /// Optionally, email of new use
    pub email: &'a str,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// data required to update them email of a user
pub struct UpdateEmail<'a> {
    /// username of the user
    pub username: &'a str,
    /// new email address of the user
    pub new_email: &'a str,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// types of credentials used as identifiers during login
pub enum Login<'a> {
    /// username as login
    Username(&'a str),
    /// email as login
    Email(&'a str),
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// type encapsulating username and hashed password of a user
pub struct NameHash {
    /// username
    pub username: String,
    /// hashed password
    pub hash: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub name: String,
}

impl Event {
    fn new(name: String) -> Self {
        Self { name }
    }

    pub fn from_str(name: &str) -> Option<Event> {
        (*EVENTS).into_iter().find(|e| e.name == name).cloned()
    }
}
lazy_static! {
    pub static ref EVENT_TYPE_CREATE: Event = Event::new("site.event.create".into());
    pub static ref EVENT_TYPE_UPDATE: Event = Event::new("site.event.update".into());
    pub static ref EVENT_TYPE_DELETE: Event = Event::new("site.event.delete".into());
    pub static ref EVENTS: [&'static Event; 3] = [
        &*EVENT_TYPE_CREATE,
        &*EVENT_TYPE_DELETE,
        &*EVENT_TYPE_UPDATE
    ];
}

struct InnerLibrepagesEvent {
    name: String,
    time: OffsetDateTime,
    pub_id: Uuid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibrePagesEvent {
    pub event_type: Event,
    pub time: OffsetDateTime,
    pub site: String,
    pub id: Uuid,
}

fn now_unix_time_stamp() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub async fn get_db(settings: &crate::settings::Settings) -> Database {
    let pool_options = PgPoolOptions::new().max_connections(settings.database.pool);
    ConnectionOptions::Fresh(Fresh {
        pool_options,
        url: settings.database.url.clone(),
        disable_logging: !settings.debug,
    })
    .connect()
    .await
    .unwrap()
}

/// map custom row not found error to DB error
pub fn map_row_not_found_err(e: sqlx::Error, row_not_found: ServiceError) -> ServiceError {
    if let sqlx::Error::RowNotFound = e {
        row_not_found
    } else {
        map_register_err(e)
    }
}

/// map postgres errors to [ServiceError](ServiceError) types
fn map_register_err(e: sqlx::Error) -> ServiceError {
    use sqlx::Error;
    use std::borrow::Cow;

    if let Error::Database(err) = e {
        if err.code() == Some(Cow::from("23505")) {
            let msg = err.message();
            println!("{}", msg);
            if msg.contains("librepages_users_name_key") {
                ServiceError::UsernameTaken
            } else if msg.contains("librepages_users_email_key") {
                ServiceError::EmailTaken
            } else {
                error!("{}", msg);
                ServiceError::InternalServerError
            }
        } else {
            ServiceError::InternalServerError
        }
    } else {
        ServiceError::InternalServerError
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::settings::Settings;

    #[test]
    fn event_names_are_unique() {
        let mut uniq = HashSet::new();
        assert!(EVENTS.into_iter().all(move |x| uniq.insert(x.name.clone())));
    }

    #[actix_rt::test]
    async fn db_works() {
        let settings = Settings::new().unwrap();
        let pool_options = PgPoolOptions::new().max_connections(1);
        let db = ConnectionOptions::Fresh(Fresh {
            pool_options,
            url: settings.database.url.clone(),
            disable_logging: !settings.debug,
        })
        .connect()
        .await
        .unwrap();
        assert!(db.ping().await);

        const EMAIL: &str = "postgresuser@foo.com";
        const EMAIL2: &str = "postgresuser2@foo.com";
        const NAME: &str = "postgresuser";
        const PASSWORD: &str = "pasdfasdfasdfadf";

        db.migrate().await.unwrap();
        let p = super::Register {
            username: NAME,
            email: EMAIL,
            hash: PASSWORD,
        };

        if db.username_exists(p.username).await.unwrap() {
            db.delete_user(p.username).await.unwrap();
            assert!(
                !db.username_exists(p.username).await.unwrap(),
                "user is deleted so username shouldn't exist"
            );
        }

        db.register(&p).await.unwrap();

        assert!(matches!(
            db.register(&p).await,
            Err(ServiceError::UsernameTaken)
        ));

        // testing get_password

        // with username
        let name_hash = db.get_password(&Login::Username(p.username)).await.unwrap();
        assert_eq!(name_hash.hash, p.hash, "user password matches");

        assert_eq!(name_hash.username, p.username, "username matches");

        // with email
        let mut name_hash = db.get_password(&Login::Email(p.email)).await.unwrap();
        assert_eq!(name_hash.hash, p.hash, "user password matches");
        assert_eq!(name_hash.username, p.username, "username matches");

        // testing get_email
        assert_eq!(db.get_email(p.username).await.unwrap(), p.email);

        // testing email exists
        assert!(
            db.email_exists(p.email).await.unwrap(),
            "user is registered so email should exist"
        );
        assert!(
            db.username_exists(p.username).await.unwrap(),
            "user is registered so username should exist"
        );

        // update password test. setting password = username
        name_hash.hash = name_hash.username.clone();
        db.update_password(&name_hash).await.unwrap();

        let name_hash = db.get_password(&Login::Username(p.username)).await.unwrap();
        assert_eq!(
            name_hash.hash, p.username,
            "user password matches with changed value"
        );
        assert_eq!(name_hash.username, p.username, "username matches");

        // update username to p.email
        assert!(
            !db.username_exists(p.email).await.unwrap(),
            "user with p.email doesn't exist. pre-check to update username to p.email"
        );
        db.update_username(p.username, p.email).await.unwrap();
        assert!(
            db.username_exists(p.email).await.unwrap(),
            "user with p.email exist post-update"
        );

        // testing update email
        let update_email = UpdateEmail {
            username: p.username,
            new_email: EMAIL2,
        };
        db.update_email(&update_email).await.unwrap();
        println!(
            "null user email: {}",
            db.email_exists(p.email).await.unwrap()
        );
        assert!(
            db.email_exists(p.email).await.unwrap(),
            "user was with empty email but email is set; so email should exist"
        );

        // deleting user
        db.delete_user(p.email).await.unwrap();
        assert!(
            !db.username_exists(p.email).await.unwrap(),
            "user is deleted so username shouldn't exist"
        );
    }

    #[actix_rt::test]
    pub async fn test_db_sites() {
        let settings = Settings::new().unwrap();
        let pool_options = PgPoolOptions::new().max_connections(1);
        let db = ConnectionOptions::Fresh(Fresh {
            pool_options,
            url: settings.database.url.clone(),
            disable_logging: !settings.debug,
        })
        .connect()
        .await
        .unwrap();
        assert!(db.ping().await);

        const EMAIL: &str = "postgresdbsiteuser@foo.com";
        const NAME: &str = "postgresdbsiteuser";
        const PASSWORD: &str = "pasdfasdfasdfadf";

        db.migrate().await.unwrap();

        // check if events are created
        for e in &*EVENTS {
            println!("Testing event type exists {}", e.name);
            assert!(db.event_type_exists(e).await.unwrap());
        }

        let p = super::Register {
            username: NAME,
            email: EMAIL,
            hash: PASSWORD,
        };

        if db.username_exists(p.username).await.unwrap() {
            db.delete_user(p.username).await.unwrap();
            assert!(
                !db.username_exists(p.username).await.unwrap(),
                "user is deleted so username shouldn't exist"
            );
        }

        db.register(&p).await.unwrap();

        let site = Site {
            site_secret: "foobar".into(),
            repo_url: "https://git.batsense.net/LibrePages/librepages.git".into(),
            branch: "librepages".into(),
            hostname: "db_works.tests.librepages.librepages.org".into(),
            owner: p.username.into(),
        };

        // test if hostname exists. Should be false
        assert!(!db.hostname_exists(&site.hostname).await.unwrap());

        // testing adding site
        db.add_site(&site).await.unwrap();

        // test if hostname exists. Should be true
        assert!(db.hostname_exists(&site.hostname).await.unwrap());

        // get site
        let db_site = db.get_site(p.username, &site.hostname).await.unwrap();
        assert_eq!(db_site, site);

        // get site by secret
        assert_eq!(
            db_site,
            db.get_site_from_secret(&site.site_secret).await.unwrap()
        );

        // list all sites owned by user
        let db_sites = db.list_all_sites(p.username).await.unwrap();
        assert_eq!(db_sites.len(), 1);
        assert_eq!(db_sites, vec![site.clone()]);

        // add event to site
        let event_id = db
            .log_event(&site.hostname, &EVENT_TYPE_CREATE)
            .await
            .unwrap();
        let event = db.get_event(&site.hostname, &event_id).await.unwrap();
        assert_eq!(event.id, event_id);
        assert_eq!(event.event_type, *EVENT_TYPE_CREATE);
        assert_eq!(event.site, site.hostname);

        assert_eq!(
            db.list_all_site_events(&site.hostname).await.unwrap(),
            vec![event]
        );

        // delete site
        db.delete_site(p.username, &site.hostname).await.unwrap();

        // test if hostname exists. Should be false
        assert!(!db.hostname_exists(&site.hostname).await.unwrap());
    }
}
