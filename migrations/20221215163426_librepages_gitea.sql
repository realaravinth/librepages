CREATE TABLE IF NOT EXISTS librepages_gitea_instances (
    url VARCHAR(3000) NOT NULL UNIQUE,
	client_id TEXT NOT NULL,
	client_secret TEXT NOT NULL,
	ID SERIAL PRIMARY KEY NOT NULL
);
