use crate::cassandra::{aliases::types::Uuid, errors::error::Error};


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

    pub fn new_integer(integer_string: String) -> Result<Self, Error>{
        let int = match integer_string.parse::<i32>(){
            Ok(value) => value,
            Err(_e) => return Err(Error::Invalid("".to_string()))
        };
        Ok(Constant::Integer(int))
    }
    pub fn new_float(float_string: String) -> Result<Self, Error>{
        let float = match float_string.parse::<f32>(){
            Ok(value) => value,
            Err(_e) => return Err(Error::Invalid("".to_string()))
        };
        Ok(Constant::Float(float))
    }
    pub fn new_boolean(bool_string: String) -> Result<Self, Error>{
        if bool_string == "TRUE"{
            Ok(Constant::Boolean(true))
        } else {
            Ok(Constant::Boolean(false))
        }
    }
    pub fn new_uuid(mut uuid: String) -> Result<Self, Error>{
        uuid.remove(8);
        uuid.remove(12);
        uuid.remove(16);
        uuid.remove(20);
        let uuid = match u128::from_str_radix(&uuid, 16){
            Ok(uuid) => uuid,
            Err(_e) => return Err(Error::SyntaxError("Esto no es un uuid".to_string()))
        };
        Ok(Constant::Uuid(uuid))
    }


    pub fn new_hex(hex_string: String) -> Result<Self, Error>{
        let hex = match i32::from_str_radix(&hex_string, hex_string.len() as u32){
            Ok(hex) => hex,
            Err(_e) => return Err(Error::SyntaxError("Esto no es un hex".to_string()))
        };
        Ok(Constant::Hex(hex))
    }

    pub fn new_blob(mut blob_string: String) -> Result<Self, Error>{
        blob_string.remove(0);
        blob_string.remove(0);
        let blob = match i32::from_str_radix(&blob_string, blob_string.len() as u32){
            Ok(blob) => blob,
            Err(_e) => return Err(Error::SyntaxError("Esto no es un blob".to_string()))
        };
        Ok(Constant::Blob(blob))
    }

    pub fn check_string(value: &str, value2: &str) -> bool{
        value == "'" && value2 == "'"
    }

    pub fn check_integer(value: &str) -> bool{
        value.parse::<i32>().is_ok()
    }

    pub fn check_float(value: &str) -> bool{
        value.parse::<f32>().is_ok()
    }

    pub fn check_boolean(value: &String) -> bool{
        value == "TRUE" || value == "FALSE"
    }

    pub fn check_uuid(value: &str) -> bool{
        for (counter, char) in value.chars().enumerate(){
            if (counter == 8 || counter == 13 && counter == 18 || counter == 23) && char != '-'{
                return false
            }
        }
        true
    }

    pub fn check_hex(value: &str) -> bool{
        i32::from_str_radix(value, value.len() as u32).is_ok()
    }

    pub fn check_blob(value: &str) -> bool{
        if !value.starts_with("0x"){
            return false
        };
        i32::from_str_radix(&value[2..], (value.len() - 2) as u32).is_ok()
    }
}

