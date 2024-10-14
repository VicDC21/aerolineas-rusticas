use super::option::Options;
use crate::parser::data_types::keyspace_name::KeyspaceName;

pub struct CreateKeyspace {
    if_not_exist: bool,
    keyspace_name: KeyspaceName,
    options: Vec<Options>,
}

impl CreateKeyspace {
    pub fn new(if_not_exist: bool, keyspace_name: KeyspaceName, options: Vec<Options>) -> Self {
        CreateKeyspace {
            if_not_exist,
            keyspace_name,
            options,
        }
    }
}
