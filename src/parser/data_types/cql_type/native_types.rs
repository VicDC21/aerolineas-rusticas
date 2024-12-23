use crate::protocol::{aliases::results::Result, errors::error::Error};

#[derive(PartialEq)]

/// Tipos de datos nativos de CQL.
#[derive(Debug)]
pub enum NativeType {
    /// Caracter de tipo ASCII.
    Ascii,

    /// Número con signo de 64 bits.
    Bigint,

    /// Bytes arbitrarios (sin validación).
    Blob,

    /// Un booleano, true o false.
    Boolean,

    /// Columna de tipo contador (número con signo de 64 bits).
    Counter,

    /// Una fecha (sin el valor de tiempo correspondiente).
    Date,

    /// Número decimal de precisión variable.
    Decimal,

    /// Número de punto flotante de 64 bits IEEE-754.
    Double,

    /// Una duración con precisión de nanosegundos.
    Duration,

    /// Número de punto flotante de 32 bits IEEE-754.
    Float,

    /// Una dirección IP, ya sea IPv4 (4 bytes de largo) o IPv6 (16 bytes de largo). No hay una constante Inet, la dirección IP debe ingresarse como un string.
    Inet,

    /// Número con signo de 32 bits.
    Int,

    /// Número con signo de 16 bits.
    SmallInt,

    /// String codificado en UTF8.
    Text,

    /// Una hora (sin el valor de fecha correspondiente) con precisión de nanosegundos.
    Time,

    /// Un timestamp (fecha y hora) con precisión de milisegundos.
    TimeStamp,

    /// Un UUID de la versión 1, generalmente utilizado como un timestamp "libre de conflictos".
    TimeUuid,

    /// Número con signo de 8 bits.
    TinyInt,

    /// Un UUID (de cualquier versión).
    Uuid,

    /// String codificado en UTF8.
    Varchar,

    /// Número entero de precisión arbitraria.
    Varint,

    /// Vector de valores de punto flotante de longitud fija, no nula y aplanada.
    Vector,
}

impl NativeType {
    /// Verifica si la lista de tokens es un tipo de dato nativo. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso estar vacía la lista.
    pub fn parse_data_type(tokens: &mut Vec<String>) -> Result<Option<NativeType>> {
        if tokens.is_empty() {
            return Err(Error::SyntaxError("Expected data type".to_string()));
        }

        let type_name = tokens.remove(0).to_lowercase();

        match type_name.as_str() {
            "ascii" => Ok(Some(NativeType::Ascii)),
            "bigint" => Ok(Some(NativeType::Bigint)),
            "blob" => Ok(Some(NativeType::Blob)),
            "boolean" => Ok(Some(NativeType::Boolean)),
            "counter" => Ok(Some(NativeType::Counter)),
            "date" => Ok(Some(NativeType::Date)),
            "decimal" => Ok(Some(NativeType::Decimal)),
            "double" => Ok(Some(NativeType::Double)),
            "duration" => Ok(Some(NativeType::Duration)),
            "float" => Ok(Some(NativeType::Float)),
            "inet" => Ok(Some(NativeType::Inet)),
            "int" => Ok(Some(NativeType::Int)),
            "smallint" => Ok(Some(NativeType::SmallInt)),
            "text" => Ok(Some(NativeType::Text)),
            "time" => Ok(Some(NativeType::Time)),
            "timestamp" => Ok(Some(NativeType::TimeStamp)),
            "timeuuid" => Ok(Some(NativeType::TimeUuid)),
            "tinyint" => Ok(Some(NativeType::TinyInt)),
            "uuid" => Ok(Some(NativeType::Uuid)),
            "varchar" => Ok(Some(NativeType::Varchar)),
            "varint" => Ok(Some(NativeType::Varint)),
            "vector" => Ok(Some(NativeType::Vector)),
            _ => Ok(None),
        }
    }
}
