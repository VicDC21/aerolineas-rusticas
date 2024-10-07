pub enum NativeType{
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
    Vector
}