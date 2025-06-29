/// Obtiene la ruta absoluta desde la raíz del workspace
pub fn get_root_path(relative_path: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent() // Si estás en un subcrate, sube un nivel
        .unwrap_or_else(|| std::path::Path::new(manifest_dir));

    workspace_root
        .join(relative_path)
        .to_string_lossy()
        .to_string()
}
