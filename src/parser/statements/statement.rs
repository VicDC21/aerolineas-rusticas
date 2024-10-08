use crate::parser::statements::ddl_statement::ddl_statement_parser::DdlStatement;
use crate::parser::statements::dml_statement::dml_statement_parser::DmlStatement;
use crate::parser::statements::role_or_permission_statement::role_or_permission_statement_parser::RoleOrPermissionStatement;
use crate::parser::statements::udt_statement::udt_statement_parser::UdtStatement;

pub enum Statement {
    DdlStatement(DdlStatement),
    DmlStatement(DmlStatement),
    RoleOrPermissionStatement(RoleOrPermissionStatement),
    UdtStatement(UdtStatement),
}
