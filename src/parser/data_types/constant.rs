use crate::cassandra::aliases::types::Uuid;


pub enum Constant {
    String(String),
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Uuid(Uuid),
    Hex(i32),
    Blob(i32),
    NULL
}

impl Constant{
    pub fn check_string(value: &String) -> bool{
        true
    }

    pub fn check_integer(value: &String) -> bool{
        value.parse::<i32>().is_ok()
    }

    pub fn check_float(value: &String) -> bool{
        value.parse::<f32>().is_ok()
    }

    pub fn check_boolean(value: &String) -> bool{
        value == "TRUE" || value == "FALSE"
    }

    pub fn check_uuid(value: &String) -> bool{
        // value.chars().nth(8) == "-" && value. == "-" && value[18] == "-" && value [23] == "-"
        // for char in value.chars(){
            
        // }
        true
    }

    pub fn check_hex(value: &String) -> bool{
        true
    }

    pub fn check_blob(value: &String) -> bool{
        true
    }



}

