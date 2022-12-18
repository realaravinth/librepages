-- Add migration script here
CREATE TABLE IF NOT EXISTS librepages_gitea_oidc_configuration (
	gitea_instance INTEGER NOT NULL references librepages_gitea_instances(ID) ON DELETE CASCADE,
    authorization_endpoint VARCHAR(3000) NOT NULL UNIQUE,
    token_endpoint VARCHAR(3000) NOT NULL UNIQUE,
    userinfo_endpoint VARCHAR(3000) NOT NULL UNIQUE,
    introspection_endpoint VARCHAR(3000) NOT NULL UNIQUE,
	ID SERIAL PRIMARY KEY NOT NULL
)
