//! Módulo para correr la interfaz.

use eframe::{run_native, NativeOptions, Result as UIResult};

use crate::interface::app::AerolineasApp;

/// Corre la aplicación.
pub fn run_app() -> UIResult<()> {
    run_native(
        "Aerolíneas App",
        NativeOptions::default(),
        Box::new(|_cc| Box::<AerolineasApp>::default()),
    )
}
