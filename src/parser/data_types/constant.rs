use crate::cassandra::errors::error::Error;
use crate::cassandra::aliases::types::{Int, Float, Uuid};

// Revisar u32 despues de mergear para no hacer conflicto
pub enum Constant {
    String(String),
    Integer(Int),
    Float(Float),
    Boolean(bool),
    Uuid(Uuid),
    Blob(Int),
    NULL,
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::String(s1), Constant::String(s2)) => s1 == s2,
            (Constant::Integer(i1), Constant::Integer(i2)) => i1 == i2,
            (Constant::Float(f1), Constant::Float(f2)) => f1 == f2,
            (Constant::Boolean(b1), Constant::Boolean(b2)) => b1 == b2,
            (Constant::Uuid(u1), Constant::Uuid(u2)) => u1 == u2,
            (Constant::Blob(b1), Constant::Blob(b2)) => b1 == b2,
            (Constant::NULL, Constant::NULL) => true,
            _ => false,
        }
    }
}

impl Constant {
    pub fn new_integer(integer_string: String) -> Result<Self, Error> {
        let int = match integer_string.parse::<Int>() {
            Ok(value) => value,
            Err(_e) => return Err(Error::Invalid("".to_string())),
        };
        Ok(Constant::Integer(int))
    }
    pub fn new_float(float_string: String) -> Result<Self, Error> {
        let float = match float_string.parse::<Float>() {
            Ok(value) => value,
            Err(_e) => return Err(Error::Invalid("".to_string())),
        };
        Ok(Constant::Float(float))
    }
    pub fn new_boolean(bool_string: String) -> Result<Self, Error> {
        if bool_string == "TRUE" {
            Ok(Constant::Boolean(true))
        } else {
            Ok(Constant::Boolean(false))
        }
    }
    pub fn new_uuid(mut uuid: String) -> Result<Self, Error> {
        uuid.remove(8);
        uuid.remove(12);
        uuid.remove(16);
        uuid.remove(20);
        let uuid = match Uuid::from_str_radix(&uuid, 16) {
            Ok(uuid) => uuid,
            Err(_e) => return Err(Error::SyntaxError("Esto no es un uuid".to_string())),
        };
        Ok(Constant::Uuid(uuid))
    }

    pub fn new_blob(mut blob_string: String) -> Result<Self, Error> {
        blob_string.remove(0);
        blob_string.remove(0);
        let blob = match Int::from_str_radix(&blob_string, 16) {
            Ok(blob) => blob,
            Err(_e) => return Err(Error::SyntaxError("Esto no es un blob".to_string())),
        };
        Ok(Constant::Blob(blob))
    }

    pub fn check_string(value: &str, value2: &str) -> bool {
        value == "'" && value2 == "'"
    }

    pub fn check_integer(value: &str) -> bool {
        value.parse::<Int>().is_ok()
    }

    pub fn check_float(value: &str) -> bool {
        value.parse::<Float>().is_ok()
    }

    pub fn check_boolean(value: &String) -> bool {
        value == "TRUE" || value == "FALSE"
    }

    pub fn check_uuid(value: &str) -> bool {
        for (counter, char) in value.chars().enumerate() {
            if (counter == 8 || counter == 13 && counter == 18 || counter == 23) && char != '-' {
                return false;
            }
        }
        true
    }

    pub fn check_hex(value: &str) -> bool {
        Int::from_str_radix(value, value.len() as u32).is_ok()
    }

    pub fn check_blob(value: &str) -> bool {
        if !value.starts_with("0x") {
            return false;
        };
        Int::from_str_radix(&value[2..], (value.len() - 2) as u32).is_ok()
    }
}
