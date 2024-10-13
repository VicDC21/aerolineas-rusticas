use crate::parser::data_types::keyspace_name::KeyspaceName;
use crate::parser::data_types::option::Options;

pub struct AlterKeyspace {
    pub if_exists: bool,
    pub name: KeyspaceName,
    pub options: Vec<Options>,
}

impl AlterKeyspace {
    pub fn new(if_exists: bool, name: KeyspaceName, options: Vec<Options>) -> Self {
        AlterKeyspace {
            if_exists,
            name,
            options,
        }
    }
}
