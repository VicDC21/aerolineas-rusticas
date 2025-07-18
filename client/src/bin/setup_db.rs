use std::{
    fs::read_to_string,
    io::Write,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

const PATH_TO_SCRIPTS: &str = "scripts/init/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Iniciando configuración automática de la base de datos...");

    let login_content = read_to_string(format!("{PATH_TO_SCRIPTS}/login.cql"))?;
    let ks_content = read_to_string(format!("{PATH_TO_SCRIPTS}/ks.cql"))?;
    let tables_content = read_to_string(format!("{PATH_TO_SCRIPTS}/tables.cql"))?;

    println!("Archivos CQL leídos correctamente.");

    // Iniciar el cliente
    let mut child = Command::new("cargo")
        .args(["run", "-p", "client"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    println!("Cliente iniciado. Esperando a que esté listo...");

    let stdin = child
        .stdin
        .as_mut()
        .ok_or("No se pudo obtener stdin del proceso")?;

    thread::sleep(Duration::from_secs(2));

    println!("Enviando queries de configuración...");

    let queries = [&login_content, &ks_content, &tables_content];

    for (i, query) in queries.iter().enumerate() {
        let query_lines: Vec<&str> = query.trim().split('\n').collect();
        for line in query_lines {
            if !line.trim().is_empty() {
                println!("Ejecutando: {}", line.trim());
                writeln!(stdin, "{}", line.trim())?;
                stdin.flush()?;
                thread::sleep(Duration::from_millis(500));
            }
        }
        println!("Grupo de queries {} completado.", i + 1);
    }

    println!("Configuración completada. Cerrando cliente...");

    writeln!(stdin, "q")?;
    stdin.flush()?;

    let output = child.wait_with_output()?;

    if output.status.success() {
        println!("✅ Base de datos configurada exitosamente!");
    } else {
        println!("❌ Error en la configuración:");
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
