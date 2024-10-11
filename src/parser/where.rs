pub struct Where {
    pub conditions: Vec<(String, String, String)>,
}

impl Where {
    pub fn new() -> Self {
        Where {
            conditions: Vec::new(),
        }
    }

    pub fn add_condition(&mut self, column: String, operator: String, value: String) {
        self.conditions.push((column, operator, value));
    }
}