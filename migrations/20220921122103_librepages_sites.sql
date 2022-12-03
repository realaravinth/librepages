CREATE TABLE IF NOT EXISTS librepages_sites (
    site_secret VARCHAR(32) NOT NULL UNIQUE,
    repo_url VARCHAR(3000) NOT NULL,
    branch TEXT NOT NULL,
    hostname VARCHAR(3000) NOT NULL UNIQUE,
    pub_id uuid NOT NULL UNIQUE,
	ID SERIAL PRIMARY KEY NOT NULL,
	owned_by INTEGER NOT NULL references librepages_users(ID) ON DELETE CASCADE
);

CREATE UNIQUE INDEX librepages_sites_site_secret ON librepages_sites(site_secret);
CREATE UNIQUE INDEX librepages_sites_site_pub_id ON librepages_sites(pub_id);
