CREATE TABLE IF NOT EXISTS 'user' (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_ts INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS user__name ON 'user'(name);

CREATE TABLE IF NOT EXISTS user_email (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    email TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES 'user'(id)
);
CREATE INDEX IF NOT EXISTS user_email__user_id_email ON user_email(user_id, email);
-- This does put in a restriction - what about the use case where a user
-- may have multiple accounts on the system and they can just use one
-- email address for all of them?  Also multiple concurrent logins at
-- the same time to toggle between normal/super user?
-- Anyway, this will be some future work, can't allow scope creep for
-- now, so leave this restriction in place until a later time.
CREATE UNIQUE INDEX IF NOT EXISTS user_email__email ON user_email(email);

-- To prevent abuse, there needs to be a system to hold new email
-- addresses to be bound to a specific user.  We don't want the system
-- to tell any client whether an incoming email address exists within
-- the system, and to do so an internal token will be provided.
CREATE TABLE IF NOT EXISTS user_email_bindreq (
    id INTEGER PRIMARY KEY NOT NULL,
    email TEXT NOT NULL,
    origin_user_id INTEGER,  -- it may or may not have a user already attached
    origin TEXT,  -- some form of hostname/IP address
    token TEXT,  -- the token to access this record
    created_ts INTEGER NOT NULL,
    -- this will disable the token and functionally log a rejection;
    -- may indicate an ongoing abusive usage by the origin
    rejected BOOLEAN
);
CREATE INDEX IF NOT EXISTS user_email_bindreq__token ON user_email_bindreq(token);

CREATE TABLE IF NOT EXISTS user_password (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    password TEXT NOT NULL,
    created_ts INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES 'user'(id)
);
-- not creating unique index to permit the use case for not reusing
-- passwords though in the implementation it's likely going to remove
-- old passwords when reset.
CREATE INDEX IF NOT EXISTS user_password__user_id ON user_email(user_id);

-- This provides a user with the given role that only becomes active for
-- any given resource at a workflow state where the role is enabled for
-- the specific action.
--
-- In effect, this grants the user the specific role in the system, and
-- the system will only grant this role to the user for applicable
-- resources.
CREATE TABLE IF NOT EXISTS user_role (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER,
    role TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES 'user'(id)
);
CREATE INDEX IF NOT EXISTS user_role__user_id ON user_role(user_id);
CREATE UNIQUE INDEX IF NOT EXISTS user_role__user_id_role ON user_role(user_id, role);

-- This grants the user a specific role at the resource.  Currently, the
-- role is a enum encoded in the underlying application; a future
-- refinement may allow this to be fully user definable.  The grant will
-- be applied through the Casbin model that implements a form of RBAC
-- when used in conjunction with the workflow policy and the resource
-- state.
CREATE TABLE IF NOT EXISTS res_grant (
    id INTEGER PRIMARY KEY NOT NULL,
    res TEXT NOT NULL,
    user_id INTEGER,
    role TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES 'user'(id)
);
CREATE INDEX IF NOT EXISTS res_grant__res ON res_grant(res);
CREATE UNIQUE INDEX IF NOT EXISTS res_grant__res_user_id_role ON res_grant(res, user_id, role);

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
    -- formerly this was split into an endpoint group and method, but
    -- rather than coupling this to HTTP, using a more generic action
    -- identifier may better describe the intent without being directly
    -- coupled to some underlying protocol; the basic action is simply
    -- an empty string.
    action TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS wf_policy__state ON wf_policy(state);

CREATE TABLE IF NOT EXISTS res_wf_state (
    res TEXT PRIMARY KEY NOT NULL,
    -- this wil be joined with wf_policy__state to derive the rbac rules
    -- to be passed onto the engine.
    state TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS res_wf_state__res ON res_wf_state(res);

-- Note that the token for the user_session here SHOULD NOT be conveyed
-- outside of the system directly (e.g. not even through cookies) - as
-- the token is equivalent to the key to become the user and the design
-- here is immutable.  An additional layer will indirectly reference the
-- token (again locally) - packages such as `tower-sessions` designed
-- with token cycling in mind to avoid session fixation should provide
-- the indirect token to this token.
-- If multiple concurrent logins per session will be implemented, the
-- actual session provider will hold multiple tokens.
CREATE TABLE IF NOT EXISTS user_session (
    token TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    origin TEXT NOT NULL,
    created_ts INTEGER NOT NULL,
    last_active_ts INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES 'user'(id)
);
CREATE INDEX IF NOT EXISTS user_session__user_id ON user_session(user_id);
