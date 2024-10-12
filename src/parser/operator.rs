pub enum Operator {
    Equal,
    Minor,
    Mayor,
    MinorEqual,
    MayorEqual,
    Distinct,
    In,
    Contains,
    ContainsKey,
}


impl Operator{
    pub fn is_operator(operator: &String) -> Option<Operator>{
        if operator == "<" {
            Some(Operator::Minor)
        } else if operator == ">" {
            Some(Operator::Mayor)
        } else if operator == "=" {
            Some(Operator::Equal)
        } else if operator == "<=" {
            Some(Operator::MinorEqual)
        } else if operator == ">=" {
            Some(Operator::MayorEqual)
        } else if operator == "!=" {
            Some(Operator::Distinct)
        } else if operator == "IN" {
            Some(Operator::In)
        } else if operator == "CONTAINS" {
            Some(Operator::Contains)
        } else if operator == "CONTAINS KEY" {
            Some(Operator::ContainsKey)
        } else {
            None
        }
    }
}