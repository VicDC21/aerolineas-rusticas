pub enum Collection{
    /// MAP '<' cql_type',' cql_type'>'
    Map,

    /// SET '<' cql_type '>'
    Set,

    /// LIST '<' cql_type'>'
    List
}