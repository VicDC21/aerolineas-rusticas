#[derive(Debug, Clone)]
pub enum IfCondition {
    Exists,
    Conditions(Vec<Condition>),
}

#[derive(Debug, Clone)]
pub enum Condition {
    Equals(String, Value),
    NotEquals(String, Value),
    GreaterThan(String, Value),
    GreaterThanOrEqual(String, Value),
    LessThan(String, Value),
    LessThanOrEqual(String, Value),
    In(String, Vec<Value>),
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<Value>),
    Identifier(String),
}

impl IfCondition {
    pub fn new_exists() -> Self {
        IfCondition::Exists
    }

    pub fn new_conditions(conditions: Vec<Condition>) -> Self {
        IfCondition::Conditions(conditions)
    }
}

impl Condition {
    pub fn column(&self) -> &str {
        match self {
            Condition::Equals(col, _) => col,
            Condition::NotEquals(col, _) => col,
            Condition::GreaterThan(col, _) => col,
            Condition::GreaterThanOrEqual(col, _) => col,
            Condition::LessThan(col, _) => col,
            Condition::LessThanOrEqual(col, _) => col,
            Condition::In(col, _) => col,
        }
    }

    pub fn value(&self) -> &Value {
        match self {
            Condition::Equals(_, val) => val,
            Condition::NotEquals(_, val) => val,
            Condition::GreaterThan(_, val) => val,
            Condition::GreaterThanOrEqual(_, val) => val,
            Condition::LessThan(_, val) => val,
            Condition::LessThanOrEqual(_, val) => val,
            Condition::In(_, _) => panic!("In condition does not have a single value"),
        }
    }
}