CREATE TABLE IF NOT EXISTS 'user' (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS user__name ON 'user'(name);

CREATE TABLE IF NOT EXISTS user_email (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    email TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS user_email__user_id_email ON user_email(user_id, email);
CREATE UNIQUE INDEX IF NOT EXISTS user_email__email ON user_email(email);

CREATE TABLE IF NOT EXISTS user_password (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    password TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
-- not creating unique index to permit the use case for not reusing
-- passwords though in the implementation it's likely going to remove
-- old passwords when reset.
CREATE INDEX IF NOT EXISTS user_password__user_id ON user_email(user_id);

-- This would group the grants for a given resource for the rbac model
-- if we really want this to be very robust, the role will be an
-- identifier, there would be annotations associating the role name to
-- the role id and then the end points may also associate, but that's
-- too complicated for what we are currently doing, so a simple
-- descriptive string token will suffice for now.
CREATE TABLE IF NOT EXISTS res_grant (
    id INTEGER PRIMARY KEY NOT NULL,
    res TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    role TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS res_grant__res ON res_grant(res);

-- Workflow state policy - each resource, on top of permission grants it
-- may hold, will also be at a specific workflow state which will also
-- provide additional permissions grants for the resource.
--
-- Unlike PMR2 which was built on top of Plone, the workflow transitions
-- and the workflow states will be hard-coded into the application as
-- there no current plans to make this a general purpose worflow engine,
-- and the main goal for now is to generate a casbin compatible policy
-- quickly.
--
-- Hopefully in the future this may be revisited so that this model may
-- be refactored into a more general purpose form.
CREATE TABLE IF NOT EXISTS wf_policy (
    id INTEGER PRIMARY KEY NOT NULL,
    -- the name of the workflow state, e.g. "published"
    state TEXT NOT NULL,
    -- the following three fields will be passed to the rbac engine.
    -- the role that will be granted for the named state, e.g. "reader"
    role TEXT NOT NULL,
    -- the endpoint group associated for the role and state, e.g. "" for
    -- the basic group, or "editor" for granting reviewer access to the
    -- resource that is under review.
    endpoint_group TEXT NOT NULL,
    -- the HTTP method associated with the endpoint group.
    method TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS wf_policy__state ON wf_policy(state);

CREATE TABLE IF NOT EXISTS res_wf_state (
    res TEXT PRIMARY KEY NOT NULL,
    -- this wil be joined with wf_policy__state to derive the rbac rules
    -- to be passed onto the engine.
    state TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS res_wf_state__res ON res_wf_state(res);
