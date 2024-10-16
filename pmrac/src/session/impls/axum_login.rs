use pmrcore::ac::session::SessionToken;
use ::axum_login::AuthUser;
use super::*;

// Normally (as seen in other implementations), the user actually refers
// to some real user entity, but in the current implementation, the
// `Session` type effectively function as a user (albeit an instance of
// them) and have our own set of APIs that can invalidate them at will.
// So while it is generally not good practice to have the hash being
// identical to the identifier, the threat of staleness of the hash is
// mitigated due to the more hands-on lifecycle management of sessions.
//
// See: https://github.com/maxcountryman/axum-login/discussions/222
impl AuthUser for Session {
    type Id = SessionToken;

    fn id(&self) -> Self::Id {
        self.session().token
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.session().token.as_bytes()
    }
}
