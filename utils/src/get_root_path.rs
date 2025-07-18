/// Obtiene la ruta absoluta desde la raíz del workspace
pub fn get_root_path(relative_path: &str) -> Result<String, &'static str> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = std::path::Path::new(manifest_dir);

    let workspace_root = match manifest_path.parent() {
        Some(parent) => parent,
        None => manifest_path, // Fallback al directorio actual si no hay parent
    };

    Ok(workspace_root
        .join(relative_path)
        .to_string_lossy()
        .to_string())
}
