use crate::cassandra::errors::error::Error;

#[derive(Clone, Eq, PartialEq, Hash)]
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

pub fn parse_data_type(tokens: &mut Vec<String>) -> Result<NativeType, Error> {
    if tokens.is_empty() {
        return Err(Error::SyntaxError("Expected data type".to_string()));
    }

    let type_name = tokens.remove(0).to_lowercase();

    match type_name.as_str() {
        "ascii" => Ok(NativeType::Ascii),
        "bigint" => Ok(NativeType::Bigint),
        "blob" => Ok(NativeType::Blob),
        "boolean" => Ok(NativeType::Boolean),
        "counter" => Ok(NativeType::Counter),
        "date" => Ok(NativeType::Date),
        "decimal" => Ok(NativeType::Decimal),
        "double" => Ok(NativeType::Double),
        "duration" => Ok(NativeType::Duration),
        "float" => Ok(NativeType::Float),
        "inet" => Ok(NativeType::Inet),
        "int" => Ok(NativeType::Int),
        "smallint" => Ok(NativeType::SmallInt),
        "text" => Ok(NativeType::Text),
        "time" => Ok(NativeType::Time),
        "timestamp" => Ok(NativeType::TimeStamp),
        "timeuuid" => Ok(NativeType::TimeUuid),
        "tinyint" => Ok(NativeType::TinyInt),
        "uuid" => Ok(NativeType::Uuid),
        "varchar" => Ok(NativeType::Varchar),
        "varint" => Ok(NativeType::Varint),
        _ => Err(Error::SyntaxError(format!(
            "Unknown data type: {}",
            type_name
        ))),
    }
}
