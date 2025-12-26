-- Subjects (people, llm, app)
CREATE TABLE IF NOT EXISTS id_subject (
  sid           text PRIMARY KEY,         -- "ubl:sid:<blake3(pubkey|user_handle)>"
  kind          text NOT NULL CHECK (kind IN ('person','llm','app')),
  display_name  text NOT NULL,
  status        text NOT NULL DEFAULT 'active',
  created_at    timestamptz NOT NULL DEFAULT now()
);

-- Credentials (passkey, ed25519, mtls)
CREATE TABLE IF NOT EXISTS id_credential (
  id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  sid           text NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  credential_kind text NOT NULL CHECK (credential_kind IN ('passkey','ed25519','mtls')),
  -- passkey
  credential_id text,           -- base64url
  public_key    bytea NOT NULL,
  sign_count    bigint DEFAULT 0,
  backup_eligible boolean,
  backup_state boolean,
  transports    text[],
  -- ed25519
  key_version   integer NOT NULL DEFAULT 1,
  -- common
  created_at    timestamptz NOT NULL DEFAULT now(),
  UNIQUE (sid, credential_kind, key_version)
);

-- Challenges (register/login)
CREATE TABLE IF NOT EXISTS id_challenge (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  kind         text NOT NULL CHECK (kind IN ('register','login')),
  sid          text REFERENCES id_subject(sid) ON DELETE SET NULL, -- NULL em register
  challenge    bytea NOT NULL,
  origin       text NOT NULL,
  expires_at   timestamptz NOT NULL,
  used         boolean NOT NULL DEFAULT false
);

-- Sessions (user and ICTE)
CREATE TABLE IF NOT EXISTS id_session (
  sid           text NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  session_id    uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  flavor        text NOT NULL CHECK (flavor IN ('user','ict')),
  scope         jsonb NOT NULL DEFAULT '{}'::jsonb,
  not_before    timestamptz NOT NULL DEFAULT now(),
  not_after     timestamptz NOT NULL,
  created_at    timestamptz NOT NULL DEFAULT now()
);

-- Agent Signing Certificates (ASC)
CREATE TABLE IF NOT EXISTS id_asc (
  asc_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  sid          text NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  public_key   bytea NOT NULL,
  scopes       jsonb NOT NULL,
  not_before   timestamptz NOT NULL,
  not_after    timestamptz NOT NULL,
  signature    bytea NOT NULL, -- assinatura pela autoridade UBL ID (Ed25519)
  created_at   timestamptz NOT NULL DEFAULT now()
);

-- Key revocations / rotation
CREATE TABLE IF NOT EXISTS id_key_revocation (
  sid          text NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  key_version  integer NOT NULL,
  revoked_at   timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (sid, key_version)
);
