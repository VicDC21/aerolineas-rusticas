use {
    chrono::Utc,
    std::{
        fmt,
        fs::{self, OpenOptions},
        io::{self, Write},
        path::{Path, PathBuf},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
}

impl Color {
    fn to_ansi(self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

// Configuración para la rotación de archivos
#[derive(Clone)]
pub struct RotationConfig {
    max_size: u64,  // Tamaño máximo del archivo en bytes
    max_files: u32, // Número máximo de archivos de respaldo
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024, // 10MB por defecto
            max_files: 5,
        }
    }
}

// Configuración para el formato de los mensajes
#[derive(Clone)]
pub struct LogFormatter {
    timestamp_format: String,
    message_template: String,
}

impl Default for LogFormatter {
    fn default() -> Self {
        Self {
            timestamp_format: "%Y-%m-%d %H:%M:%S".to_string(),
            message_template: "[{level}] [{timestamp}]: {message}".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Logger {
    log_file: PathBuf,
    min_level: LogLevel,
    rotation_config: RotationConfig,
    formatter: LogFormatter,
}

impl Logger {
    /// Crea una nueva instancia del logger con configuración personalizada
    pub fn new(
        dir: &Path,
        ip: &str,
        min_level: LogLevel,
        rotation_config: Option<RotationConfig>,
        formatter: Option<LogFormatter>,
    ) -> Result<Self, LoggerError> {
        // Nos aseguramos de que el directorio existe
        if !dir.is_dir() {
            fs::create_dir_all(dir).map_err(LoggerError::from)?;
        }

        let log_file = dir.join(format!("node_{}.log", ip.replace(":", "_")));

        // Creamos el archivo si no existe
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .map_err(LoggerError::from)?;

        let rotation_config = match rotation_config {
            Some(config) => {
                if config.max_files == 0 {
                    return Err(LoggerError::InvalidPath(
                        "El número máximo de archivos debe ser mayor a cero".to_string(),
                    ));
                }
                config
            }
            None => RotationConfig::default(),
        };

        Ok(Self {
            log_file,
            min_level,
            rotation_config,
            formatter: formatter.unwrap_or_default(),
        })
    }

    /// Registra un mensaje si su nivel es igual o superior al nivel mínimo configurado
    pub fn log(&self, level: LogLevel, msg: &str, color: Option<Color>) -> Result<(), LoggerError> {
        // Verificamos si debemos registrar este nivel
        if level < self.min_level {
            return Ok(());
        }

        // Verificamos si necesitamos rotar el archivo
        self.rotate_if_needed()?;

        // Formateamos el mensaje
        let timestamp = Utc::now()
            .format(&self.formatter.timestamp_format)
            .to_string();
        let log_msg = self
            .formatter
            .message_template
            .replace("{level}", &level.to_string())
            .replace("{timestamp}", &timestamp)
            .replace("{message}", msg);
        let log_msg = format!("{}\n", log_msg);

        // Si hay color especificado, lo aplicamos para stdout
        if let Some(color) = color {
            let colored_msg = format!("{}{}\x1b[0m", color.to_ansi(), log_msg);
            print!("{}", colored_msg);
            io::stdout().flush().map_err(LoggerError::from)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .map_err(LoggerError::from)?;

        file.write_all(log_msg.as_bytes())
            .map_err(LoggerError::from)?;
        file.flush().map_err(LoggerError::from)?;

        Ok(())
    }

    /// Implementa la rotación de archivos cuando sea necesario
    fn rotate_if_needed(&self) -> Result<(), LoggerError> {
        let metadata = fs::metadata(&self.log_file)?;

        if metadata.len() > self.rotation_config.max_size {
            // Movemos los archivos existentes
            for i in (1..self.rotation_config.max_files).rev() {
                let current = self.log_file.with_extension(format!("log.{}", i));
                let next = self.log_file.with_extension(format!("log.{}", i + 1));

                if current.exists() {
                    if i == self.rotation_config.max_files - 1 {
                        fs::remove_file(current)?;
                    } else {
                        fs::rename(current, next)?;
                    }
                }
            }

            fs::rename(&self.log_file, self.log_file.with_extension("log.1"))?;

            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.log_file)?;
        }

        Ok(())
    }

    pub fn debug(&self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::Debug, msg, Some(Color::Blue))
    }

    pub fn info(&self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::Info, msg, Some(Color::Green))
    }

    pub fn warning(&self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::Warning, msg, Some(Color::Yellow))
    }

    pub fn error(&self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::Error, msg, Some(Color::Red))
    }
}

#[derive(Debug)]
pub enum LoggerError {
    IoError(std::io::Error),
    InvalidPath(String),
    CompressionError(String),
}

impl std::fmt::Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerError::IoError(e) => write!(f, "Error de E/S: {}", e),
            LoggerError::InvalidPath(msg) => write!(f, "Ruta inválida: {}", msg),
            LoggerError::CompressionError(msg) => write!(f, "Error de compresión: {}", msg),
        }
    }
}

impl std::error::Error for LoggerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoggerError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for LoggerError {
    fn from(err: std::io::Error) -> Self {
        LoggerError::IoError(err)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, std::sync::Arc, tempfile::TempDir};

    // Función auxiliar para crear un directorio temporal y un logger para pruebas
    fn setup_test_logger() -> (TempDir, Logger) {
        let temp_dir = TempDir::new().expect("Error al crear directorio temporal");
        let logger = Logger::new(
            temp_dir.path(),
            "127.0.0.1:8080",
            LogLevel::Trace,
            None,
            None,
        )
        .expect("Error al crear el logger");

        (temp_dir, logger)
    }

    #[test]
    fn test_logger_creation() {
        let (temp_dir, _) = setup_test_logger();
        let log_file = temp_dir.path().join("node_127.0.0.1_8080.log");
        assert!(log_file.exists(), "El archivo de log no fue creado");
    }

    #[test]
    fn test_log_levels() {
        let (temp_dir, logger) = setup_test_logger();

        logger
            .debug("Mensaje debug")
            .expect("Error en el log de mensaje de debug");
        logger
            .info("Mensaje info")
            .expect("Error en el log de mensaje de info");
        logger
            .warning("Mensaje warning")
            .expect("Error en el log de mensaje de warning");
        logger
            .error("Mensaje error")
            .expect("Error en el log de mensaje de error");

        let log_content = fs::read_to_string(temp_dir.path().join("node_127.0.0.1_8080.log"))
            .expect("Error al leer el archivo de log");

        assert!(log_content.contains("DEBUG"));
        assert!(log_content.contains("INFO"));
        assert!(log_content.contains("WARNING"));
        assert!(log_content.contains("ERROR"));
    }

    #[test]
    fn test_log_level_filtering() {
        let temp_dir = TempDir::new().expect("Error al crear directorio temporal");

        let logger = Logger::new(
            temp_dir.path(),
            "127.0.0.1:8080",
            LogLevel::Info,
            None,
            None,
        )
        .expect("Error al crear el logger");

        logger
            .debug("No debería aparecer")
            .expect("Error en el log de mensaje de debug");
        logger
            .info("Debería aparecer")
            .expect("Error en el log de mensaje de info");
        logger
            .warning("Debería aparecer")
            .expect("Error en el log de mensaje de warning");

        // Leemos el contenido del archivo
        let log_content = fs::read_to_string(temp_dir.path().join("node_127.0.0.1_8080.log"))
            .expect("Error al leer el archivo de log");

        // Verificamos el filtrado
        assert!(!log_content.contains("DEBUG"));
        assert!(log_content.contains("INFO"));
        assert!(log_content.contains("WARNING"));
    }

    #[test]
    fn test_custom_formatter() {
        let temp_dir = TempDir::new().expect("Error al crear directorio temporal");

        let formatter = LogFormatter {
            timestamp_format: "%H:%M:%S".to_string(),
            message_template: "TEST-{level}: {message}".to_string(),
        };

        let logger = Logger::new(
            temp_dir.path(),
            "127.0.0.1:8080",
            LogLevel::Info,
            None,
            Some(formatter),
        )
        .expect("Error al crear el logger");

        logger
            .info("Mensaje de prueba")
            .expect("Error al registrar mensaje");

        let log_content = fs::read_to_string(temp_dir.path().join("node_127.0.0.1_8080.log"))
            .expect("Error al leer el archivo de log");

        assert!(log_content.contains("TEST-INFO"));
        assert!(!log_content.contains("[INFO]"));
    }

    #[test]
    fn test_rotation() {
        let temp_dir = TempDir::new().expect("Error al crear directorio temporal");

        let rotation_config = RotationConfig {
            max_size: 100,
            max_files: 3,
        };

        let logger = Logger::new(
            temp_dir.path(),
            "127.0.0.1:8080",
            LogLevel::Info,
            Some(rotation_config),
            None,
        )
        .expect("Error al crear el logger");

        for i in 0..10 {
            logger
                .info(&format!("Mensaje largo de prueba número {}", i))
                .expect("Error al registrar mensaje");
        }

        assert!(temp_dir.path().join("node_127.0.0.1_8080.log").exists());
        assert!(temp_dir.path().join("node_127.0.0.1_8080.log.1").exists());
        assert!(temp_dir.path().join("node_127.0.0.1_8080.log.2").exists());
        assert!(!temp_dir.path().join("node_127.0.0.1_8080.log.4").exists());
    }

    #[test]
    fn test_invalid_directory() {
        let result = Logger::new(
            Path::new("/path/that/definitely/does/not/exist"),
            "127.0.0.1:8080",
            LogLevel::Info,
            None,
            None,
        );

        assert!(result.is_err());
        match result {
            Err(LoggerError::IoError(_)) => (),
            _ => panic!("Se esperaba un IoError"),
        }
    }

    #[test]
    fn test_concurrent_logging() {
        let (temp_dir, logger) = setup_test_logger();
        let logger = Arc::new(logger);
        let mut handles = vec![];

        // Creamos múltiples hilos que escriben logs simultáneamente
        for i in 0..10 {
            let logger_clone = Arc::clone(&logger);
            let handle = std::thread::spawn(move || {
                for j in 0..10 {
                    logger_clone
                        .info(&format!("Mensaje del hilo {} número {}", i, j))
                        .expect("Error al registrar mensaje");
                }
            });
            handles.push(handle);
        }

        // Esperamos a que todos los hilos terminen
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Verificamos que se hayan escrito todos los mensajes
        let log_content = fs::read_to_string(temp_dir.path().join("node_127.0.0.1_8080.log"))
            .expect("Error al leer el archivo de log");

        // Deberían haber 100 mensajes en total (10 hilos * 10 mensajes)
        let message_count = log_content.lines().count();
        assert_eq!(
            message_count, 100,
            "No se registraron todos los mensajes esperados"
        );
    }
}
