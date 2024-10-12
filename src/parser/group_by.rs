use super::data_types::identifier::Identifier;

pub struct GroupBy {
    columns: Vec<Identifier>
}


impl GroupBy{
    pub fn new(columns: Vec<Identifier>) -> Self{
        GroupBy{columns}
    }
}