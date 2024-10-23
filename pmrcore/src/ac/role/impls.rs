use std::{
    fmt,
    str::FromStr,
};
use crate::error::ValueError;
use super::*;

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", <&'static str>::from(*self))
    }
}

impl From<Role> for String {
    fn from(role: Role) -> String {
        format!("{role}")
    }
}

impl From<Role> for &'static str {
    fn from(role: Role) -> &'static str {
        match role {
            Role::Manager => "manager",
            Role::Owner => "owner",
            Role::Editor => "editor",
            Role::Reviewer => "reviewer",
            Role::Reader => "reader",
            Role::Undefined => "undefined",
        }
    }
}

impl FromStr for Role {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "manager" => Ok(Role::Manager),
            "owner" => Ok(Role::Owner),
            "editor" => Ok(Role::Editor),
            "reviewer" => Ok(Role::Reviewer),
            "reader" => Ok(Role::Reader),
            // Undefined,
            s => Err(ValueError::Unsupported(s.to_string())),
        }
    }
}

impl Roles {
    pub fn contains(&self, role: Role) -> bool {
        self.0 & role == role
    }
}

impl FromIterator<Role> for Roles {
    fn from_iter<I: IntoIterator<Item=Role>>(iter: I) -> Self {
        Roles(iter.into_iter().collect())
    }
}

impl IntoIterator for Roles {
    type Item = Role;
    type IntoIter = enumset::EnumSetIter<Role>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(feature = "clap")]
mod clap {
    use ::clap::{
        ValueEnum,
        builder::PossibleValue,
    };
    use super::*;

    impl ValueEnum for Role {
        fn value_variants<'a>() -> &'a [Self] {
            &[Role::Manager, Role::Owner, Role::Editor, Role::Reviewer, Role::Reader]
        }

        fn to_possible_value(&self) -> Option<PossibleValue> {
            match self {
                Role::Manager => Some(PossibleValue::new("manager")),
                Role::Owner => Some(PossibleValue::new("owner")),
                Role::Editor => Some(PossibleValue::new("editor")),
                Role::Reviewer => Some(PossibleValue::new("reviewer")),
                Role::Reader => Some(PossibleValue::new("reader")),
                Role::Undefined => None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::*;
    use crate::error::ValueError;

    #[test]
    fn smoke_role() -> anyhow::Result<()> {
        // sample of standard conversions
        assert_eq!(Role::Manager.to_string(), "manager");
        assert_eq!(Role::Manager, Role::from_str("manager")?);
        assert_eq!(Role::Owner.to_string(), "owner");
        assert_eq!(Role::Owner, Role::from_str("owner")?);

        // error conversion
        assert!(Role::from_str("undefined").is_err());
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

    #[test]
    fn smoke_roles() -> anyhow::Result<()> {
        let role_set = Roles::from_iter([
            Role::Owner,
            Role::Reader,
        ]);
        assert!(role_set.contains(Role::Owner));
        assert!(!role_set.contains(Role::Manager));
        assert_eq!(
            &role_set.into_iter().collect::<Vec<_>>(),
            &[Role::Owner, Role::Reader],
        );
        Ok(())
    }
}
