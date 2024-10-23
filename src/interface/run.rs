//! Módulo para correr la interfaz.

use eframe::{run_native, NativeOptions};

use crate::interface::app::AerolineasApp;
use crate::protocol::{aliases::results::Result, errors::error::Error};

/// Corre la aplicación.
pub fn run_app() -> Result<()> {
    if let Err(err) = run_native(
        "Aerolíneas App",
        NativeOptions::default(),
        Box::new(|cc| {
            Ok(Box::<AerolineasApp>::new(AerolineasApp::new(
                cc.egui_ctx.clone(),
            )))
        }),
    ) {
        return Err(Error::ServerError(format!(
            "Ha ocurrido un error al correr la aplicación:\n\n{}",
            err
        )));
    }
    Ok(())
}
