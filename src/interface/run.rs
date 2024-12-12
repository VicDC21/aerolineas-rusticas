//! Módulo para correr la interfaz.

use {
    crate::{
        interface::app::AerolineasApp,
        protocol::{aliases::results::Result, errors::error::Error::ServerError},
    },
    eframe::{
        egui::ViewportBuilder,
        {run_native, NativeOptions},
    },
    std::io::{Error as IoError, ErrorKind},
};

/// Corre la aplicación.
pub fn run_app() -> Result<()> {
    if let Err(err) = run_native(
        "Aerolíneas App",
        NativeOptions {
            viewport: ViewportBuilder::default().with_maximized(true),
            ..Default::default()
        },
        Box::new(|cc| match AerolineasApp::new(cc.egui_ctx.clone()) {
            Ok(app) => Ok(Box::new(app)),
            Err(err) => {
                let error = IoError::new(ErrorKind::Other, err.to_string());
                Err(Box::new(error))
            }
        }),
    ) {
        return Err(ServerError(format!(
            "Ha ocurrido un error al correr la aplicación:\n\n{}",
            err
        )));
    }
    Ok(())
}
