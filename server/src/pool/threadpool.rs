//! Módulo para un _ThreadPool_.

use {
    crate::pool::{job::JobType, worker::Worker},
    protocol::{aliases::results::Result, errors::error::Error},
    std::sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
};

/// Un _ThreadPool_ intenta reutilizar los hilos disponibles para realizar
/// tareas paralelas.
pub struct ThreadPool {
    /// La lista de _workers_ a nuestra disposición.
    workers: Vec<Worker>,

    /// El canal de envío de tareas.
    sender: Sender<JobType>,
}

impl ThreadPool {
    /// Crea una nueva instancia del _ThreadPool_.
    pub fn new(workers: Vec<Worker>, sender: Sender<JobType>) -> Self {
        Self { workers, sender }
    }

    /// Intenta construir una instancia de _ThreadPool_, con la cantidad de hilos indicada.
    pub fn build(n_threads: usize) -> Result<Self> {
        if n_threads == 0 {
            // Un usize no puede ser negativo
            return Err(Error::ServerError(format!(
                "{} no es un número de hilos válidos para el ThreadPool.",
                n_threads
            )));
        }

        let (sender, receiver) = channel::<JobType>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::<Worker>::with_capacity(n_threads);
        for i in 0..n_threads {
            workers.push(Worker::build(i, Arc::clone(&receiver))?);
        }

        Ok(Self::new(workers, sender))
    }

    /// Ejecuta una tarea en un hilo disponible.
    pub fn execute<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        let job = Box::new(f);
        if let Err(send_err) = self.sender.send(JobType::NewTask(job)) {
            return Err(Error::ServerError(format!(
                "Error mandando código a worker:\n\n{}",
                send_err
            )));
        }
        Ok(())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            let _ = self.sender.send(JobType::Exit);
        }
        // Los workers se dropean solos al salir de scope.
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new(Vec::new(), channel().0)
    }
}
