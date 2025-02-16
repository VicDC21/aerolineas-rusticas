//! Módulo para un _Job_.

use protocol::aliases::results::Result;

/// Un _Job_ representa una tarea en ejecución de parte de un [Worker](crate::pool::worker::Worker).
pub type Job = Box<dyn FnOnce() -> Result<()> + Send + 'static>;

/// Un tipo específico de tarea a realizar.
#[derive(Default)]
pub enum JobType {
    /// Ejecutar una nueva tarea.
    NewTask(Job),

    /// Terminar la ejecución del [Worker](crate::pool::worker::Worker).
    #[default]
    Exit,
}
