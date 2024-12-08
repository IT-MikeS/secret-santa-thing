CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY,
    name TEXT,
    creator TEXT,
    is_generated BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS members (
    group_id TEXT,
    user_id TEXT,
    name TEXT,
    is_creator BOOLEAN,
    FOREIGN KEY(group_id) REFERENCES groups(id),
    UNIQUE(group_id, name),
    UNIQUE(group_id, user_id)
);

CREATE TABLE IF NOT EXISTS assignments (
    group_id TEXT,
    giver_id TEXT,
    receiver_id TEXT,
    FOREIGN KEY(group_id) REFERENCES groups(id),
    UNIQUE(group_id, giver_id)
);