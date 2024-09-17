use std::{
    fmt,
    str::FromStr,
};
use crate::error::ValueError;
use super::Role;

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for Role {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Manager" => Ok(Role::Manager),
            "Owner" => Ok(Role::Owner),
            "Editor" => Ok(Role::Editor),
            "Reviewer" => Ok(Role::Reviewer),
            "Reader" => Ok(Role::Reader),
            // Undefined,
            s => Err(ValueError::Unsupported(s.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::Role;
    use crate::error::ValueError;

    #[test]
    fn smoke() -> anyhow::Result<()> {
        // sample of standard conversions
        assert_eq!(Role::Manager.to_string(), "Manager");
        assert_eq!(Role::Manager, Role::from_str("Manager")?);
        assert_eq!(Role::Owner.to_string(), "Owner");
        assert_eq!(Role::Owner, Role::from_str("Owner")?);

        // error conversion
        assert!(Role::from_str("Undefined").is_err());
        assert!(matches!(
            Role::from_str("no_such_role")
                .expect_err("should be an error"),
            ValueError::Unsupported(s) if s == "no_such_role".to_string(),
        ));

        // infallable conversion
        assert_eq!(
            Role::from_str("no_such_role")
                .unwrap_or_default(),
            Role::Undefined,
        );
        Ok(())
    }
}
