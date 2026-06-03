/// SQL migration for creating all CogniFlow tables.
pub const CREATE_TABLES: &str = r#"
CREATE TABLE IF NOT EXISTS prompts (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    folder_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived INTEGER NOT NULL DEFAULT 0,
    current_version INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS prompt_versions (
    id TEXT PRIMARY KEY NOT NULL,
    prompt_id TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    content TEXT NOT NULL,
    change_summary TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    intent TEXT NOT NULL DEFAULT '',
    folder_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS workflow_steps (
    id TEXT PRIMARY KEY NOT NULL,
    workflow_id TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    title TEXT NOT NULL,
    instruction TEXT NOT NULL DEFAULT '',
    prompt_id TEXT,
    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE,
    FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    depth INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS prompt_tags (
    prompt_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (prompt_id, tag_id),
    FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS prompt_categories (
    prompt_id TEXT NOT NULL,
    category_id TEXT NOT NULL,
    PRIMARY KEY (prompt_id, category_id),
    FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS intents (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS relationships (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0
);

CREATE TABLE IF NOT EXISTS embeddings (
    entity_id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    vector BLOB NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_prompts_folder ON prompts(folder_id);
CREATE INDEX IF NOT EXISTS idx_prompts_archived ON prompts(archived);
CREATE INDEX IF NOT EXISTS idx_prompt_versions_prompt ON prompt_versions(prompt_id);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_workflow ON workflow_steps(workflow_id);
CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id);
CREATE INDEX IF NOT EXISTS idx_relationships_source ON relationships(source_id);
CREATE INDEX IF NOT EXISTS idx_relationships_target ON relationships(target_id);
"#;
