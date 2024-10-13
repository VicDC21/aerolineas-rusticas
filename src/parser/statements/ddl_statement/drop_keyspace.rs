use crate::parser::data_types::keyspace_name::KeyspaceName;

pub struct DropKeyspace {
    pub if_exists: bool,
    pub name: KeyspaceName,
}

impl DropKeyspace {
    pub fn new(if_exists: bool, name: KeyspaceName) -> Self {
        DropKeyspace { if_exists, name }
    }
}
