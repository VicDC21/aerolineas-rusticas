use super::quoted_name::QuotedName;
use super::unquoted_name::UnquotedName;

pub enum KeyspaceName{
    UnquotedName(UnquotedName),
    QuotedName(UnquotedName)
}