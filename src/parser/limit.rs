pub struct Limit {
    limit: i32
    // bind _marker
}


impl Limit{
    pub fn new(limit: i32) ->Self{
        Limit{limit}
    }
}