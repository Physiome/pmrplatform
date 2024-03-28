CREATE TABLE IF NOT EXISTS exposure (
    id INTEGER PRIMARY KEY NOT NULL,
    workspace_id INTEGER NOT NULL,
    workspace_tag_id INTEGER,
    commit_id TEXT NOT NULL,  -- this is actually duplicate with tag
    created_ts INTEGER NOT NULL,
    default_file_id INTEGER,
    FOREIGN KEY(workspace_id) REFERENCES workspace(id),
    FOREIGN KEY(workspace_tag_id) REFERENCES workspace_tag(id),
    FOREIGN KEY(default_file_id) REFERENCES exposure_file(id)
);

CREATE INDEX IF NOT EXISTS exposure__workspace_id ON exposure(workspace_id);
CREATE INDEX IF NOT EXISTS exposure__workspace_id_commit_id ON exposure(workspace_id, commit_id);

-- basedir / exposure_id / workspace_file_path = taskdir_root?
CREATE TABLE IF NOT EXISTS exposure_file (
    id INTEGER PRIMARY KEY NOT NULL,
    exposure_id INTEGER NOT NULL,
    workspace_file_path TEXT NOT NULL,
    default_view_id INTEGER,
    FOREIGN KEY(exposure_id) REFERENCES exposure(id),
    FOREIGN KEY(default_view_id) REFERENCES exposure_file_view(id)
);

CREATE INDEX IF NOT EXISTS exposure_file__exposure_id ON exposure_file(exposure_id);
CREATE UNIQUE INDEX IF NOT EXISTS exposure_file__exposure_id_workspace_file_path ON exposure_file(exposure_id, workspace_file_path);

CREATE TABLE IF NOT EXISTS exposure_file_view (
    id INTEGER PRIMARY KEY NOT NULL,
    exposure_file_id INTEGER NOT NULL,
    -- This tracks which view task template is assigned to this
    -- particular view
    view_task_template_id INTEGER NOT NULL,
    -- This is additional information that is only associated with the
    -- underlying task, and for now in the name of simplicity this is
    -- only a one-to-one relationship defined here.
    --
    -- The other entity will have a reverse reference to this id, but
    -- that is not the canonical reference as it won't have its foreign
    -- key defined to refer to this entity.
    exposure_file_view_task_id INTEGER,
    -- The view_key functions as the suffix to access the data presented
    -- at that /{view} end point, and is set when the task spawned via
    -- the task template completes.
    -- The reason for this duplication is to anchor the view_key at the
    -- time of the original task completion, the fragility result from
    -- this non-linkage may be useful to ensure task is rerun should
    -- there are migration needed.
    -- New view_task_template should be created, then the task rerun
    -- to generate the new view_key may be the migration?
    -- While the goal is to ensure existing things keep working, there
    -- may be a situation where the third-party thing breaks completely
    -- and this might be the way to indicate that.
    -- TODO determine how to make view_key UNIQUE, and policy on how
    -- this can be empty (e.g. on initialization/deletion)
    view_key TEXT,
    -- Should the reference to the view_task_template is updated, this
    -- need to be set, so to discriminate against tasks that have been
    -- completed prior to the update.
    updated_ts INTEGER NOT NULL,
    -- the views are then implemented by the underlying framework
    FOREIGN KEY(exposure_file_id) REFERENCES exposure_file(id),
    FOREIGN KEY(view_task_template_id) REFERENCES view_task_template(id),
    FOREIGN KEY(exposure_file_view_task_id) REFERENCES exposure_file_view_task(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS exposure_file_view__exposure_file_id_view_task_template_id ON exposure_file_view(exposure_file_id, view_task_template_id);

-- To ensure that there is a one-to-one binding of the file view to the
-- underlying task_template.
CREATE UNIQUE INDEX IF NOT EXISTS exposure_file_view__exposure_file_id_task_view_task_id ON exposure_file_view(exposure_file_id, exposure_file_view_task_id);
-- TODO determine if the view_key also need to be unique for the given
-- exposure_file_id - as multiple tasks defined for another profile can
-- be choosen arbitrarily, it may be possible that this results in a
-- conflict on which view gets assigned.

-- The task template is attached to the exposure file to store the work-
-- in-progress template usage for the file, in order to facilitate the
-- declaration of the intent to use those view task templates for the
-- generation of the views.
CREATE TABLE IF NOT EXISTS exposure_file_view_task_template (
    id INTEGER PRIMARY KEY NOT NULL,
    exposure_file_id INTEGER NOT NULL,
    view_task_template_id INTEGER NOT NULL,
    FOREIGN KEY(exposure_file_id) REFERENCES exposure_file(id),
    FOREIGN KEY(view_task_template_id) REFERENCES view_task_template(id)
);

-- TODO implement a table for temporary answers so that all work-in-
-- progress answers can be stored, including non-validated submissions.
-- Given the typical usage as a key-value store per each id, the schema
-- could be as simple as an id - json.

CREATE TABLE IF NOT EXISTS exposure_file_view_task (
    id INTEGER PRIMARY KEY NOT NULL,
    -- This records the previous attempts to spawn tasks.  Should refer
    -- to an existing view at the time of creation, but given this is a
    -- record of some view, this won't have a foreign key reference.
    --
    -- The reverse direction will have a foreign key declaration to this
    -- entity.
    exposure_file_view_id INTEGER NOT NULL,
    -- To ensure the task has the correct reference.
    --
    -- Originally there was an idea that a single view could be composed
    -- together by multiple tasks, but that's complexity that should be
    -- exported to the task itself - if the task need to run multiple
    -- things, then provide the entry point there and let that task do
    -- the spawning of additional tasks.
    view_task_template_id INTEGER NOT NULL,
    -- This references the task that resides on the pmrtqs platform.
    -- Currently, this design does NOT track all past tasks that have
    -- been spawned using this task entry, but rather it acts like an
    -- intention to spawn one of these tasks using the template.
    -- The rationale to not focus on tracking of this is because the
    -- task is ultimately managed by the pmrtqs system (as opposed to
    -- this one, pmrapp), and given that all the task entities reside
    -- there, it has the separate set of metadata that are managed
    -- under that system - they will have the lower level information
    -- that loosely correlate to the ones here at a particular point
    -- in time (e.g. the working_dir at the time for that task), which
    -- may actually be irrelevant after some time.
    -- Should the design need to changed to track this detailed info,
    -- the anchoring id should be on exposure_file_task_template(id)
    -- and a separate entry to point to the result exposure_file_view(id)
    -- that got produced.
    -- For the prototype, this seems too much work already for something
    -- that is getting rather deep into the weeds so this is getting
    -- punted to be decided later
    -- *** So for now, this is a big entry that tracks a particular view
    -- with its latest associated task/template that created it, not to
    -- serve as a comprehensive log of every creation process that
    -- happened.
    task_id INTEGER,
    -- Track the creation timestamp to ensure that this record is still
    -- relevant - should it be before the updated_ts then this task
    -- should be considered invalidated.
    created_ts INTEGER,
    -- No need to store when the underlying task was done, just mark it
    -- as ready when the underlying task is completed successfully.
    ready BOOLEAN NOT NULL,
    FOREIGN KEY(view_task_template_id) REFERENCES view_task_template(id)
);

-- The view_task_template table tracks each of the available task
-- template that may be used to generate a view.  There will be
-- duplicate views as the same end point may provide different data
-- representation for the variety of data being tracked.
CREATE TABLE IF NOT EXISTS view_task_template (
    id INTEGER PRIMARY KEY NOT NULL,
    -- Note that this is not unique - different files could have the
    -- same view_key, and they will have different task_template_id
    -- that will differentiate them due to the variety of to be
    -- supported data and view combinations.  This view_key will be
    -- assigned to the exposure_file_view entry whe the underlying
    -- task is completed.
    view_key TEXT NOT NULL,
    -- The description appropriate for the view vs. the task template
    -- that will be used on the selected incoming file.
    description TEXT NOT NULL,
    -- This references the task_template that resides on the pmrtqs
    -- platform.
    task_template_id INTEGER NOT NULL,
    updated_ts INTEGER NOT NULL
);

-- A profile is a collection of relevant view_task_templates - this
-- is not prefixed with the exposure_file resource despite being
-- initially designed for them - reason being is that these should be
-- generalized models.
CREATE TABLE IF NOT EXISTS profile (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL
    -- TODO maybe this will require an additional discriminant for
    -- what the given profile is for, though it might just end up
    -- being multi-purpose.
);
CREATE UNIQUE INDEX IF NOT EXISTS profile__profile_title ON profile(title);

-- Likewise for views - these could be generalized for other resource
-- types.  Rather than calling this `profile_view_task_template`, the
-- task_template is only a means to the end - which is producing the
-- view itself.
-- Table ending as a plural to indicate a simple joiner table that
-- intends to be an intermediate.
CREATE TABLE IF NOT EXISTS profile_views (
    id INTEGER PRIMARY KEY NOT NULL,
    profile_id INTEGER NOT NULL,
    view_task_template_id INTEGER NOT NULL,
    FOREIGN KEY(profile_id) REFERENCES profile(id),
    FOREIGN KEY(view_task_template_id) REFERENCES view_task_template(id)
);
-- Note that no tracking of when the profile got views extended or
-- removed - this can cause a previously defined exposure view not
-- matching with the current profile.  This may have consequences
-- for the next table.

CREATE UNIQUE INDEX IF NOT EXISTS profile_views__profile_id_view_task_template_id ON profile_views(profile_id, view_task_template_id);

-- The profile selected for a given exposure file.
CREATE TABLE IF NOT EXISTS exposure_file_profile (
    id INTEGER PRIMARY KEY NOT NULL,
    exposure_file_id INTEGER NOT NULL,
    profile_id INTEGER NOT NULL,
    FOREIGN KEY(profile_id) REFERENCES profile(id),
    FOREIGN KEY(exposure_file_id) REFERENCES exposure_file(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS exposure_file_profile__exposure_file_id ON exposure_file_profile(exposure_file_id);
CREATE INDEX IF NOT EXISTS exposure_file_profile__exposure_file_id_profile_id ON exposure_file_profile(exposure_file_id, profile_id);

-- The user input for a given exposure_file_profile.
CREATE TABLE IF NOT EXISTS exposure_file_profile_input (
    id INTEGER PRIMARY KEY NOT NULL,
    exposure_file_profile_id INTEGER NOT NULL,
    -- This is the id for the arg from pmrtqs.
    arg_id INTEGER NOT NULL,
    input TEXT NOT NULL,
    FOREIGN KEY(exposure_file_profile_id) REFERENCES exposure_file_profile(id)
);
CREATE INDEX IF NOT EXISTS exposure_file_profile_input__exposure_file_profile_id ON exposure_file_profile_input(exposure_file_profile_id);
CREATE UNIQUE INDEX IF NOT EXISTS exposure_file_profile_input__exposure_file_profile_id_arg_id ON exposure_file_profile_input(exposure_file_profile_id, arg_id);
