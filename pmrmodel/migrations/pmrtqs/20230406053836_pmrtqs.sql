-- Task runner should be on a separate project
-- allocate tasks through API?

CREATE TABLE IF NOT EXISTS task_basedir (
    id INTEGER PRIMARY KEY NOT NULL,
    path TEXT NOT NULL
);

-- workdir and tmpdir implied?
-- need a final dir, to ensure that only workdir entries be moved once process complete
-- also this prevents loss of data on job rerun that fail

-- TODO also all of this lies independant of workspace data - this may be a good thing
-- to keep the project independant, but integration becomes an interesting problem...

CREATE TABLE IF NOT EXISTS task (
    id INTEGER PRIMARY KEY NOT NULL,
    bin_path TEXT NOT NULL,
    pid INTEGER,
    start_ts INTEGER,
    stop_ts INTEGER,
    exit_status INTEGER,
    -- TODO figure out if we link the above?
    -- task_basedir_id NOT NULL,
    -- FOREIGN KEY(task_basedir_id) REFERENCES task_basedir(id)
    basedir TEXT NOT NULL
);

CREATE INDEX task__pid ON task(pid);

CREATE TABLE IF NOT EXISTS task_arg (
    id INTEGER PRIMARY KEY NOT NULL,
    task_id INTEGER NOT NULL,
    arg TEXT NOT NULL,
    FOREIGN KEY(task_id) REFERENCES task(id)
);

CREATE INDEX task_arg__task_id ON task_arg(task_id);


CREATE TABLE IF NOT EXISTS task_template (
    id INTEGER PRIMARY KEY NOT NULL,
    bin_path TEXT NOT NULL,
    -- version_id only identifies the program version, we are not going to deal
    -- with the program's own dependency tree - NixOS implemented that so not
    -- reinventing/replicating that work here.
    version_id TEXT NOT NULL,
    created_ts INTEGER NOT NULL,
    -- if the following is present, it has been finalized and ready for use
    final_task_template_arg_id INTEGER,
    superceded_by_id INTEGER
);

-- how do we generate a UI from this?

CREATE TABLE IF NOT EXISTS task_template_arg (
    id INTEGER PRIMARY KEY NOT NULL,
    task_template_id INTEGER NOT NULL,
    flag TEXT,     -- this is for the flag argument that precede this

    -- whether or not the flag is joined with the supplied argument,
    -- e.g. those ending with = or CMake styled flags.
    flag_joined BOOLEAN NOT NULL,

    prompt TEXT,   -- NULL implies auto?
    'default' TEXT,  -- if no prompt and default, this is required?
    choice_fixed BOOLEAN NOT NULL, -- if choice is fixed, user can't edit
    choice_source TEXT, -- if NULL, ignore usage of choices
                        -- if empty string, use table below
                        -- otherwise, programmatically defined?
    -- fixed args vs generated args vs user supplied args
    -- what about calculated args?
    FOREIGN KEY(task_template_id) REFERENCES task_template(id)
);

CREATE TABLE IF NOT EXISTS task_template_arg_choice (
    id INTEGER PRIMARY KEY NOT NULL,
    task_template_arg_id INTEGER NOT NULL,
    to_arg TEXT,          -- the value this gets mapped to
    label TEXT NOT NULL,  -- the user facing label
    FOREIGN KEY(task_template_arg_id) REFERENCES task_template_arg(id)
);

CREATE INDEX task_template_arg_choice__task_template_arg_id ON task_template_arg_choice(task_template_arg_id);

-- profiles will be a collection of task templates?
-- exposure file will point to a collection of task templates, but what about relation to profiles?

-- need to determine how the basedir map to exposure/file/
-- how much should be stored about paths?
