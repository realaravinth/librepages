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

use sqlx::postgres::PgPoolOptions;
use sqlx::types::time::OffsetDateTime;
//use sqlx::types::Json;
use sqlx::ConnectOptions;
use sqlx::PgPool;

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
                //.map_err(|e| DBError::DBError(Box::new(e)))?
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
        //.map_err(|e| DBError::DBError(Box::new(e)))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

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
    }
}
