use super::{expression::Expression, operator::Operator, relation::Relation};

pub struct Where {
    pub expression: Option<Box<Expression>>,
}

impl Where {
    pub fn new(expression: Option<Box<Expression>>) -> Self {
        Where { expression }
    }

    // pub fn add_condition(&mut self, relation: Relation) {
    //     self.relations.push(relation);
    // }
}
