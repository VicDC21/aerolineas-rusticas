//! Módulo para un _worker_ que procesa tareas.

use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{Builder, JoinHandle},
};

use crate::protocol::{aliases::results::Result, errors::error::Error};
use crate::server::pool::job::JobType;

/// El tipo de hilo a usar en un _worker_.
pub type WorkerHandle = JoinHandle<Result<()>>;
/// El alias para el receptor de tareas.
pub type JobReceiver = Arc<Mutex<Receiver<JobType>>>;

/// Un _worker_ recibe tareas y las ejecuta en el único hilo que tiene.
pub struct Worker {
    /// El ID del hilo.
    id: usize,

    /// El hilo en sí.
    thread: Option<WorkerHandle>,
}

impl Worker {
    /// Crea una nueva instancia del _worker_.
    pub fn new(id: usize, thread: Option<WorkerHandle>) -> Self {
        Self { id, thread }
    }

    /// Intenta crear una nueva instancia del _worker_.
    pub fn build(id: usize, receiver: JobReceiver) -> Result<Self> {
        let builder = Builder::new().name(format!("worker_{}", id));
        let thread = match builder.spawn(move || {
            loop {
                match receiver.lock() {
                    Err(poison_err) => {
                        return Err(Error::ServerError(format!("Se detectó un lock envenenado en el worker ({}):\n\n{}", id, poison_err)));
                    },
                    Ok(lock) => {
                        match lock.recv() {
                            Err(recv_err) => {
                                return Err(Error::ServerError(format!("Ocurrió un error al recibir una tarea en el worker ({}):\n\n{}", id, recv_err)));
                            },
                            Ok(job_type) => {
                                match job_type {
                                    JobType::NewTask(job) => {
                                        job()?;
                                    },
                                    JobType::Exit => {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }) {
            Ok(created) => created,
            Err(thread_err) => {
                return Err(Error::ServerError(format!(
                    "Error creando hilo para worker ({}):\n\n{}",
                    id, thread_err
                )));
            }
        };

        Ok(Self::new(id, Some(thread)))
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
