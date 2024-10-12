use super::unquoted_name::UnquotedName;

pub enum KeyspaceName {
    UnquotedName(UnquotedName),
    QuotedName(UnquotedName),
}

impl Default for KeyspaceName {
    fn default() -> Self {
        KeyspaceName::UnquotedName(UnquotedName::default())
    }
}
