use std::cmp::Ordering;

use crate::protocol::{
    aliases::types::{Double, Int, Uuid},
    errors::error::Error,
};

// Revisar u32 despues de mergear para no hacer conflicto

#[derive(Debug)]
/// constant::= string | integer | double | boolean | uuid | blob | NULL
pub enum Constant {
    /// ''' (any character where ' can appear if doubled)+ '''.
    String(String),

    /// re('-?[0-9]+'). Es un i32 normalito.
    Integer(Int),

    /// re('-?[0-9]+(.[0-9]*)?([eE][+-]?[0-9+])?') | NAN | INFINITY. Es un f64, con eso alcanza para representar las posibilidades.
    Double(Double),

    /// TRUE | FALSE
    Boolean(bool),

    ///hex\{8}-hex\{4}-hex\{4}-hex\{4}-hex\{12}. Son 5 numeros hexa, cada uno del tamaño indicado.
    Uuid(Uuid),

    /// '0' ('x' | 'X') hex+. Numero hexa pero con prefijo '0x'
    Blob(Int),

    /// Null
    NULL,
}

impl Constant {
    /// TODO: Desc básica
    pub fn get_value(&self) -> String {
        match self {
            Constant::String(s) => s.to_string(),
            Constant::Integer(i) => i.to_string(),
            Constant::Float(f) => f.to_string(),
            Constant::Boolean(b) => b.to_string(),
            Constant::Uuid(u) => u.to_string(),
            Constant::Blob(b) => b.to_string(),
            Constant::NULL => "NULL".to_string(),
        }
    }

    /// TODO: Desc básica
    pub fn is_constant(lista: &mut Vec<String>) -> Result<Option<Constant>, Error> {
        if lista.len() > 2 && Constant::check_string(&lista[0], &lista[2]) {
            lista.remove(0);
            let string = Constant::String(lista.remove(0));
            lista.remove(0);
            return Ok(Some(string));
        } else if !lista.is_empty() {
            if Constant::check_integer(&lista[0]) {
                let integer_string: String = lista.remove(0);
                let int = Constant::new_integer(integer_string)?;
                return Ok(Some(int));
            } else if Constant::check_double(&lista[0]) {
                let double_string = lista.remove(0);
                let double = Constant::new_double(double_string)?;
                return Ok(Some(double));
            } else if Constant::check_boolean(&lista[0]) {
                let bool = lista.remove(0);
                let bool = Constant::new_boolean(bool)?;
                return Ok(Some(bool));
            } else if Constant::check_uuid(&lista[0]) {
                let uuid = lista.remove(0);
                let uuid = Constant::new_uuid(uuid)?;
                return Ok(Some(uuid));
            } else if Constant::check_blob(&lista[0]) {
                let blob = Constant::new_blob(lista.remove(0))?;
                return Ok(Some(blob));
            }
        }
        Ok(None)
    }

    fn new_integer(integer_string: String) -> Result<Self, Error> {
        let int = match integer_string.parse::<Int>() {
            Ok(value) => value,
            Err(_e) => return Err(Error::Invalid("".to_string())),
        };
        Ok(Constant::Integer(int))
    }
    fn new_double(double_string: String) -> Result<Self, Error> {
        let double = match double_string.parse::<Double>() {
            Ok(value) => value,
            Err(_e) => return Err(Error::Invalid("".to_string())),
        };
        Ok(Constant::Double(double))
    }
    fn new_boolean(bool_string: String) -> Result<Self, Error> {
        if bool_string == "TRUE" {
            Ok(Constant::Boolean(true))
        } else {
            Ok(Constant::Boolean(false))
        }
    }
    fn new_uuid(mut uuid: String) -> Result<Self, Error> {
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

    fn new_blob(mut blob_string: String) -> Result<Self, Error> {
        blob_string.remove(0);
        blob_string.remove(0);
        let blob = match Int::from_str_radix(&blob_string, 16) {
            Ok(blob) => blob,
            Err(_e) => return Err(Error::SyntaxError("Esto no es un blob".to_string())),
        };
        Ok(Constant::Blob(blob))
    }

    fn check_string(value: &str, value2: &str) -> bool {
        value == "'" && value2 == "'"
    }

    fn check_integer(value: &str) -> bool {
        value.parse::<Int>().is_ok()
    }

    fn check_double(value: &str) -> bool {
        value.parse::<Double>().is_ok()
    }

    fn check_boolean(value: &String) -> bool {
        value == "TRUE" || value == "FALSE"
    }

    fn check_uuid(value: &str) -> bool {
        if value.len() != 36 {
            return false;
        }
        for (counter, char) in value.chars().enumerate() {
            if (counter == 8 || counter == 13 && counter == 18 || counter == 23) && char != '-' {
                return false;
            }
        }
        if !Constant::check_hex(&value[0..8])
            || !Constant::check_hex(&value[9..13])
            || !Constant::check_hex(&value[14..18])
            || !Constant::check_hex(&value[19..23])
            || !Constant::check_hex(&value[24..36])
        {
            return false;
        }
        true
    }

    fn check_hex(value: &str) -> bool {
        Int::from_str_radix(value, value.len() as u32).is_ok()
    }

    fn check_blob(value: &str) -> bool {
        if !value.starts_with("0x") {
            return false;
        };
        Int::from_str_radix(&value[2..], (value.len() - 2) as u32).is_ok()
    }

    /// Devuelve el valor de la constante como un String.
    pub fn get_value_as_string(&self) -> String {
        match self {
            Constant::String(value) => value.to_string(),
            Constant::Integer(value) => value.to_string(),
            Constant::Double(value) => value.to_string(),
            Constant::Boolean(value) => value.to_string(),
            Constant::Uuid(value) => value.to_string(),
            Constant::Blob(value) => value.to_string(),
            Constant::NULL => "".to_string(),
        }
    }
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::String(s1), Constant::String(s2)) => s1 == s2,
            (Constant::Integer(i1), Constant::Integer(i2)) => i1 == i2,
            (Constant::Double(f1), Constant::Double(f2)) => f1 == f2,
            (Constant::Boolean(b1), Constant::Boolean(b2)) => b1 == b2,
            (Constant::Uuid(u1), Constant::Uuid(u2)) => u1 == u2,
            (Constant::Blob(b1), Constant::Blob(b2)) => b1 == b2,
            (Constant::NULL, Constant::NULL) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Constant::Integer(i1), Constant::Integer(i2)) => i1.partial_cmp(i2),
            (Constant::Double(f1), Constant::Double(f2)) => f1.partial_cmp(f2),
            (Constant::String(s1), Constant::String(s2)) => s1.partial_cmp(s2),
            (Constant::Boolean(b1), Constant::Boolean(b2)) => b1.partial_cmp(b2),
            (Constant::Uuid(u1), Constant::Uuid(u2)) => u1.partial_cmp(u2),
            (Constant::Blob(b1), Constant::Blob(b2)) => b1.partial_cmp(b2),
            (Constant::NULL, Constant::NULL) => Some(Ordering::Equal),
            (_, _) => None,
        }
    }
}
