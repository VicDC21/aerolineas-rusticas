use super::unquoted_name::UnquotedName;

pub enum KeyspaceName {
    UnquotedName(UnquotedName),
    QuotedName(UnquotedName),
}
