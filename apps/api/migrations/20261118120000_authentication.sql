CREATE TYPE account_provider AS ENUM ('credential', 'google');
CREATE TYPE session_client AS ENUM ('web', 'mobile');
CREATE TYPE verification_purpose AS ENUM ('email_verification', 'password_reset');
CREATE TYPE email_dispatch_kind AS ENUM ('email_verification', 'password_reset');
CREATE TYPE email_outbox_status AS ENUM ('pending', 'failed');

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    email text NOT NULL UNIQUE CHECK (email = lower(email)),
    email_verified boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE accounts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider account_provider NOT NULL,
    provider_account_id text,
    password_hash text CHECK (provider <> 'credential' OR password_hash IS NOT NULL),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, provider),
    UNIQUE (provider, provider_account_id)
);

CREATE TABLE sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    client session_client NOT NULL,
    refresh_token_hash bytea NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    ip_address inet,
    user_agent text CHECK (char_length(user_agent) <= 512),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX sessions_user_id_idx ON sessions (user_id);
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at);

CREATE TABLE verifications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email text NOT NULL CHECK (email = lower(email)),
    purpose verification_purpose NOT NULL,
    token_hash bytea NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX verifications_expires_at_idx ON verifications (expires_at);
CREATE INDEX verifications_email_purpose_consumed_at_idx
    ON verifications (email, purpose, consumed_at);

CREATE TABLE email_dispatches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email text NOT NULL CHECK (email = lower(email)),
    kind email_dispatch_kind NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX email_dispatches_created_at_idx ON email_dispatches (created_at);
CREATE INDEX email_dispatches_email_kind_created_at_idx
    ON email_dispatches (email, kind, created_at);

CREATE TABLE email_outbox (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email text NOT NULL CHECK (email = lower(email)),
    subject text NOT NULL,
    html_body text,
    text_body text,
    status email_outbox_status NOT NULL DEFAULT 'pending',
    attempts integer NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 5),
    next_attempt_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (CASE status
        WHEN 'pending' THEN html_body IS NOT NULL AND text_body IS NOT NULL
        ELSE html_body IS NULL AND text_body IS NULL
    END)
);

CREATE INDEX email_outbox_next_attempt_at_idx
    ON email_outbox (next_attempt_at) WHERE status = 'pending';
CREATE INDEX email_outbox_expires_at_idx ON email_outbox (expires_at);
