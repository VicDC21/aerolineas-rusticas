//! Librería de la aplicación de simulación de vuelos.

/// Módulo de la interfaz de línea de comandos.
pub mod cli;
/// Módulo de manejo de conexión de cliente.
pub mod connection;
/// Módulo de simulación de vuelos.
pub mod flight_simulator;
/// Módulo de inicialización de la simulación.
pub mod initializer;
/// Módulo de manejo de envío de datos.
pub mod sender;
/// Modulo de pruebas unitarias.
pub mod sim_tests;
/// Módulo de actualización de vuelos.
pub mod updater;
/// Módulo de utilidades.
pub mod utils;
