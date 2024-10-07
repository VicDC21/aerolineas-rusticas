pub enum Statement{
    DdlStatement,
    DmlStatement,
    SecondaryIndexStatement,
    MaterializedViewStatement,
    RoleOrPermissionStatement,
    UdfStatement,
    UdtStatement,
    TriggerStatement
}