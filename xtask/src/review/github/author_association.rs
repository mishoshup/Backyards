use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorAssociation {
    Absent,
    Owner,
    Member,
    Collaborator,
    Contributor,
    FirstTimer,
    FirstTimeContributor,
    Mannequin,
    NoAssociation,
}

impl AuthorAssociation {
    #[must_use]
    pub const fn is_owner(self) -> bool {
        matches!(self, Self::Owner)
    }
}

impl FromStr for AuthorAssociation {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "" => Ok(Self::Absent),
            "OWNER" => Ok(Self::Owner),
            "MEMBER" => Ok(Self::Member),
            "COLLABORATOR" => Ok(Self::Collaborator),
            "CONTRIBUTOR" => Ok(Self::Contributor),
            "FIRST_TIMER" => Ok(Self::FirstTimer),
            "FIRST_TIME_CONTRIBUTOR" => Ok(Self::FirstTimeContributor),
            "MANNEQUIN" => Ok(Self::Mannequin),
            "NONE" => Ok(Self::NoAssociation),
            other => Err(format!("unknown author association {other}")),
        }
    }
}
