CREATE TYPE email_outbox_status AS ENUM ('pending', 'failed');

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
    CHECK ((status = 'pending') = (html_body IS NOT NULL AND text_body IS NOT NULL))
);
CREATE INDEX ON email_outbox (next_attempt_at) WHERE status = 'pending';
CREATE INDEX ON email_outbox (expires_at);
