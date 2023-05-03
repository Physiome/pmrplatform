CREATE TABLE IF NOT EXISTS exposure (
    id INTEGER PRIMARY KEY NOT NULL,
    workspace_id INTEGER NOT NULL,
    workspace_tag_id INTEGER NOT NULL,
    commit_id TEXT NOT NULL,  -- this is actually duplicate with tag
    created INTEGER NOT NULL,
    root_exposure_file_id INTEGER,  -- TODO figure out how to do this link to the exposure_file table.
    FOREIGN KEY(workspace_id) REFERENCES workspace(id),
    FOREIGN KEY(workspace_tag_id) REFERENCES workspace_tag(id)
);

CREATE INDEX IF NOT EXISTS exposure__workspace_id ON exposure(workspace_id);
CREATE INDEX IF NOT EXISTS exposure__workspace_id_commit_id ON exposure(workspace_id, commit_id);

-- basedir / exposure_id / workspace_file_path = taskdir_root?
CREATE TABLE IF NOT EXISTS exposure_file (
    id INTEGER PRIMARY KEY NOT NULL,
    exposure_id INTEGER NOT NULL,
    workspace_file_path TEXT NOT NULL,
    default_view TEXT,
    FOREIGN KEY(exposure_id) REFERENCES exposure(id)
);

CREATE INDEX IF NOT EXISTS exposure_file__exposure_id ON exposure_file(exposure_id);
CREATE INDEX IF NOT EXISTS exposure_file__exposure_id_workspace_file_path ON exposure_file(exposure_id, workspace_file_path);

CREATE TABLE IF NOT EXISTS exposure_file_view (
    id INTEGER PRIMARY KEY NOT NULL,
    exposure_file_id INTEGER NOT NULL,
    view_key TEXT NOT NULL,  -- the suffix to get to the view
    -- the views are then implemented by the underlying framework
    FOREIGN KEY(exposure_file_id) REFERENCES exposure_file(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS exposure_file_view__exposure_file_id_view_key ON exposure_file_view(exposure_file_id, view_key);

-- each file view can have multiple tasks
CREATE TABLE IF NOT EXISTS exposure_file_view_tasks (
    id INTEGER PRIMARY KEY NOT NULL,
    exposure_file_view_id INTEGER NOT NULL,
    task_id INTEGER,
    FOREIGN KEY(exposure_file_view_id) REFERENCES exposure_file_view(id)
);
