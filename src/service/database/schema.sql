CREATE TABLE IF NOT EXISTS chat_sessions (
    -- uuid
    id           TEXT primary key,
    -- full ChatSession proto message
    data         BLOB NOT NULL,
	-- unix seconds (UTC)
    created_at   INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
	-- unix seconds (UTC)
	updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_events (
    -- uuid
    id           TEXT primary key,
	-- uuid
    session_id   TEXT NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    -- full ChatEvent proto message
    data         BLOB NOT NULL,
    created_at   INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
