use version_compare::Version;

enum Operator {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
}

pub struct VersionRequirement {
    op: Operator,
    version: String,
}

impl VersionRequirement {
    fn new(op: Operator, version: String) -> Self {
        VersionRequirement { op, version }
    }

    pub fn parse_requirement(req: &str) -> Self {
        if let Some(rest) = req.strip_prefix(">=") {
            return Self::new(Operator::Ge, rest.to_string());
        }
        if let Some(rest) = req.strip_prefix("<=") {
            return Self::new(Operator::Le, rest.to_string());
        }
        if let Some(rest) = req.strip_prefix(">") {
            return Self::new(Operator::Gt, rest.to_string());
        }
        if let Some(rest) = req.strip_prefix("<") {
            return Self::new(Operator::Lt, rest.to_string());
        }
        if let Some(rest) = req.strip_prefix("==") {
            return Self::new(Operator::Eq, rest.to_string());
        }

        // Default to equality if no operator specified
        Self::new(Operator::Eq, req.to_string())
    }

    pub fn matches(&self, dep_version: &str) -> bool {
        let dep_version = match Version::from(dep_version) {
            Some(v) => v,
            None => return false,
        };

        let req_version = match Version::from(&self.version) {
            Some(v) => v,
            None => return false,
        };

        match self.op {
            Operator::Gt => dep_version > req_version,
            Operator::Ge => dep_version >= req_version,
            Operator::Lt => dep_version < req_version,
            Operator::Le => dep_version <= req_version,
            Operator::Eq => dep_version == req_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirement() {
        let req = VersionRequirement::parse_requirement(">1.2.3");
        assert_eq!(matches!(req.op, Operator::Gt), true);
        assert_eq!(req.version, "1.2.3");

        let req = VersionRequirement::parse_requirement(">=2.0.0");
        assert_eq!(matches!(req.op, Operator::Ge), true);
        assert_eq!(req.version, "2.0.0");

        let req = VersionRequirement::parse_requirement("<3.0.0");
        assert_eq!(matches!(req.op, Operator::Lt), true);
        assert_eq!(req.version, "3.0.0");

        let req = VersionRequirement::parse_requirement("<=4.5.6");
        assert_eq!(matches!(req.op, Operator::Le), true);
        assert_eq!(req.version, "4.5.6");

        let req = VersionRequirement::parse_requirement("==1.0.0");
        assert_eq!(matches!(req.op, Operator::Eq), true);
        assert_eq!(req.version, "1.0.0");

        let req = VersionRequirement::parse_requirement("7.8.9"); // no operator
        assert_eq!(matches!(req.op, Operator::Eq), true);
        assert_eq!(req.version, "7.8.9");
    }
}
