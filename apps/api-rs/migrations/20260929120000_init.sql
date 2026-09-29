CREATE TYPE account_provider AS ENUM ('credential', 'google');
CREATE TYPE verification_purpose AS ENUM ('email_verification', 'password_reset');
CREATE TYPE email_dispatch_kind AS ENUM ('email_verification', 'password_reset');
CREATE TYPE session_client AS ENUM ('web', 'mobile');

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    email text NOT NULL UNIQUE,
    email_verified boolean NOT NULL DEFAULT false,
    image text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE accounts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider account_provider NOT NULL,
    provider_account_id text,
    password_hash text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_account_id),
    UNIQUE (user_id, provider)
);

CREATE TABLE sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    refresh_token_hash text NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    ip_address inet,
    user_agent text,
    client session_client NOT NULL DEFAULT 'web',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON sessions (user_id);
CREATE INDEX ON sessions (expires_at);

CREATE TABLE verifications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email text NOT NULL,
    purpose verification_purpose NOT NULL,
    token_hash text NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON verifications (expires_at);
CREATE INDEX ON verifications (email, purpose, consumed_at);

CREATE TABLE email_dispatches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email text NOT NULL,
    kind email_dispatch_kind NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON email_dispatches (created_at);
CREATE INDEX ON email_dispatches (email, kind, created_at);
