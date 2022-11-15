CREATE TABLE IF NOT EXISTS librepages_deploy_event_type (
	name VARCHAR(30) NOT NULL UNIQUE,
	ID SERIAL PRIMARY KEY NOT NULL
);

CREATE UNIQUE INDEX librepages_deploy_event_name_index ON librepages_deploy_event_type(name);

CREATE TABLE IF NOT EXISTS librepages_site_deploy_events (
	site INTEGER NOT NULL references librepages_sites(ID) ON DELETE CASCADE,
	event_type INTEGER NOT NULL references librepages_deploy_event_type(ID),
	time timestamptz NOT NULL,
    pub_id uuid NOT NULL UNIQUE,
	ID SERIAL PRIMARY KEY NOT NULL
);
