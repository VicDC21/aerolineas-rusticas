use super::{expression::Expression, relation::Relation};

pub struct And {
    first_relation: Box<Expression>,
    second_relation: Box<Expression>,
}

impl And {
    pub fn new(first_relation: Box<Expression>, second_relation: Box<Expression>) -> Self {
        And {
            first_relation,
            second_relation,
        }
    }
}
