use crate::cassandra::errors::error::Error;

pub enum RoleOrPermissionStatement {
    CreateRoleStatement,
    AlterRoleStatement,
    DropRoleStatement,
    GrantRoleStatement,
    RevokeRoleStatement,
    ListRolesStatement,
    GrantPermissionStatement,
    RevokePermissionStatement,
    ListPermissionsStatement,
    CreateUserStatement,
    AlterUserStatement,
    DropUserStatement,
    ListUsersStatement,
}

pub fn role_or_permission_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    if let Some(_x) = create_role_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::CreateRoleStatement));
    } else if let Some(_x) = alter_role_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::AlterRoleStatement));
    } else if let Some(_x) = drop_role_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::DropRoleStatement));
    } else if let Some(_x) = grant_role_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::GrantRoleStatement));
    } else if let Some(_x) = revoke_role_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::RevokeRoleStatement));
    } else if let Some(_x) = list_roles_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::ListRolesStatement));
    } else if let Some(_x) = grant_permission_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::GrantPermissionStatement));
    } else if let Some(_x) = revoke_permission_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::RevokePermissionStatement));
    } else if let Some(_x) = list_permissions_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::ListPermissionsStatement));
    } else if let Some(_x) = create_user_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::CreateUserStatement));
    } else if let Some(_x) = alter_user_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::AlterUserStatement));
    } else if let Some(_x) = drop_user_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::DropUserStatement));
    } else if let Some(_x) = list_users_statement(_lista, _index)? {
        return Ok(Some(RoleOrPermissionStatement::ListUsersStatement));
    }
    Ok(None)
}

pub fn create_role_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn alter_role_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn drop_role_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn grant_role_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn revoke_role_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn list_roles_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn grant_permission_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn revoke_permission_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn list_permissions_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn create_user_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn alter_user_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn drop_user_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}

pub fn list_users_statement(
    _lista: &mut [String],
    _index: usize,
) -> Result<Option<RoleOrPermissionStatement>, Error> {
    Ok(None)
}
