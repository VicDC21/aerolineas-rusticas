use crate::cassandra::errors::error::Error;

#[derive(PartialEq)]

/// Native types are the basic data types that are supported by Cassandra.
pub enum NativeType {
    /// ASCII character string
    Ascii,

    /// 64-bit signed long
    Bigint,

    /// Arbitrary bytes (no validation)
    Blob,

    /// Either true or false
    Boolean,

    /// Counter column (64-bit signed value). See counters for details.
    Counter,

    /// A date (with no corresponding time value). See dates below for details.
    Date,

    /// Variable-precision decimal
    Decimal,

    /// 64-bit IEEE-754 floating point
    Double,

    /// A duration with nanosecond precision. See durations below for details.
    Duration,

    /// 32-bit IEEE-754 floating point
    Float,

    /// An IP address, either IPv4 (4 bytes long) or IPv6 (16 bytes long). Note that there is no inet constant, IP address should be input as strings.
    Inet,

    /// 32-bit signed int
    Int,

    /// 16-bit signed int
    SmallInt,

    /// UTF8 encoded string
    Text,

    /// A time (with no corresponding date value) with nanosecond precision. See times below for details.
    Time,

    /// A timestamp (date and time) with millisecond precision. See timestamps below for details.
    TimeStamp,

    /// Version 1 UUID, generally used as a “conflict-free” timestamp. Also see timeuuid-functions.
    TimeUuid,

    /// 8-bit signed int
    TinyInt,

    /// A UUID (of any version)
    Uuid,

    /// UTF8 encoded string
    Varchar,

    /// Arbitrary-precision integer
    Varint,

    /// A fixed length non-null, flattened array of float values CASSANDRA-18504 added this data type to Cassandra 5.0.
    Vector,
}

impl NativeType {
    /// Parse a data type from a list of tokens.
    pub fn parse_data_type(tokens: &mut Vec<String>) -> Result<Option<NativeType>, Error> {
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
