use super::quoted_identifier::QuotedIdentifier;
use super::unquoted_identifier::UnquotedIdentifier;

pub enum Identifier {
    UnquotedIdentifier(UnquotedIdentifier),
    QuotedIdentifier(QuotedIdentifier),
}
