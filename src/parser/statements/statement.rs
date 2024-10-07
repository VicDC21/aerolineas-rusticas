use crate::parser::statements::ddl_statement::ddl_statement_parser::DdlStatement;
use crate::parser::statements::dml_statement::dml_statement_parser::DmlStatement;
use crate::parser::statements::materialized_view_statement::materialized_view_statement_parser::MaterializedViewStatement;
use crate::parser::statements::role_or_permission_statement::role_or_permission_statement_parser::RoleOrPermissionStatement;
use crate::parser::statements::secondary_index_statement::secondary_index_statement_parser::SecondaryIndexStatement;
use crate::parser::statements::trigger_statement::trigger_statement_parser::TriggerStatement;
use crate::parser::statements::udf_statement::udf_statement_parser::UdfStatement;
use crate::parser::statements::udt_statement::udt_statement_parser::UdtStatement;

pub enum Statement {
    DdlStatement(DdlStatement),
    DmlStatement(DmlStatement),
    SecondaryIndexStatement(SecondaryIndexStatement),
    MaterializedViewStatement(MaterializedViewStatement),
    RoleOrPermissionStatement(RoleOrPermissionStatement),
    UdfStatement(UdfStatement),
    UdtStatement(UdtStatement),
    TriggerStatement(TriggerStatement),
}
