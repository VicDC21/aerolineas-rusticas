use {
    chrono::Utc,
    std::{
        fs::OpenOptions,
        io::{self, Write},
        path::{Path, PathBuf},
    },
};

#[derive(Debug, Clone)]
enum Level {
    Info(Color),
    Warning,
    Error,
}

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

#[derive(Debug, Clone)]
pub struct Logger {
    log_file: PathBuf,
}

impl Logger {
    /// Crea una nueva instancia de `Logger` y un archivo de log en el directorio especificado.
    ///
    /// # Parametros
    /// - `dir`: Ruta al directorio donde se creará el archivo de log.
    /// - `ip`: La dirección IP del nodo que está creando el log.
    ///
    /// # Devuelve
    /// Una nueva instancia de `Logger` si la operación fue exitosa.
    pub fn new(dir: &Path, ip: &str) -> Result<Self, LoggerError> {
        match dir.is_dir() {
            true => std::fs::create_dir_all(dir).map_err(LoggerError::from)?,
            false => {
                return Err(LoggerError::InvalidPath(
                    "La ruta proporcionada no es un directorio".into(),
                ));
            }
        }

        let log_file = dir.join(format!("node_{}.log", ip.replace(":", "_")));

        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file)
            .map_err(LoggerError::from)?;

        Ok(Logger { log_file })
    }

    /// Registra un mensaje informativo.
    ///
    /// # Parametros
    /// - `msg`: El mensaje informativo a registrar.
    /// - `color`: El color del mensaje en la consola.
    /// - `to_stdout`: Si se debe registrar el mensaje en la consola.
    pub fn info(&self, msg: &str, color: Color, to_stdout: bool) -> Result<(), LoggerError> {
        self.log(Level::Info(color), msg, to_stdout)
    }

    /// Registra un mensaje de advertencia.
    ///
    /// # Parametros
    /// - `msg`: El mensaje de advertencia a registrar.
    /// - `to_stdout`: Si se debe registrar el mensaje en la consola.
    pub fn warn(&self, msg: &str, to_stdout: bool) -> Result<(), LoggerError> {
        self.log(Level::Warning, msg, to_stdout)
    }

    /// Registra un mensaje de error.
    ///
    /// # Parametros
    /// - `msg`: El mensaje de error a registrar.https://github.com/alendavies/rustic-airlines.githttps://github.com/alendavies/rustic-airlines.git
    /// - `to_stdout`: Si se debe registrar el mensaje en la consola.
    pub fn error(&self, msg: &str, to_stdout: bool) -> Result<(), LoggerError> {
        self.log(Level::Error, msg, to_stdout)
    }

    fn log(&self, level: Level, msg: &str, to_stdout: bool) -> Result<(), LoggerError> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_msg = match &level {
            Level::Info(_) => format!("[INFO] [{}]: {}\n", timestamp, msg),
            Level::Warning => format!("[WARNING] [{}]: {}\n", timestamp, msg),
            Level::Error => format!("[ERROR] [{}]: {}\n", timestamp, msg),
        };

        if to_stdout {
            let colored_msg = match &level {
                Level::Info(color) => format!("{}{}\x1b[0m", color.to_ansi(), log_msg),
                Level::Warning => format!("\x1b[93m{}\x1b[0m", log_msg),
                Level::Error => format!("\x1b[91m{}\x1b[0m", log_msg),
            };
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
}

#[derive(Debug)]
pub enum LoggerError {
    IoError(std::io::Error),
    InvalidPath(String),
}

impl std::fmt::Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerError::IoError(e) => write!(f, "Error de E/S: {}", e),
            LoggerError::InvalidPath(msg) => write!(f, "Ruta inválida: {}", msg),
        }
    }
}

impl std::error::Error for LoggerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoggerError::IoError(e) => Some(e),
            LoggerError::InvalidPath(_) => None,
        }
    }
}

impl From<std::io::Error> for LoggerError {
    fn from(err: std::io::Error) -> Self {
        LoggerError::IoError(err)
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            log_file: PathBuf::from("default.log"),
        }
    }
}
