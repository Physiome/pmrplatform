pub struct Credentials {
    pub authorization: Authorization,
    pub origin: String,
}

#[non_exhaustive]
pub enum Authorization {
    LoginPassword(String, String),
    Token(String),
}

mod impls;
