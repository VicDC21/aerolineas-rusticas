//! Módulo para paneles de la interfaz.

use eframe::egui::{Color32, Context, Frame, Margin, RichText, SidePanel};

use crate::data::airports::Airport;

/// Muestra por un panel lateral los detalles del aeropuerto actualmente seleccionado.
pub fn airport_info(ctx: &Context, selected_airport: &Option<Airport>) {
    let panel_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(66, 66, 66, 200),
        inner_margin: Margin::ZERO,
        ..Default::default()
    };
    let info_panel = SidePanel::left("airport_info")
        .resizable(false)
        .exact_width(ctx.screen_rect().width() / 3.0)
        .frame(panel_frame);
    info_panel.show_animated(ctx, selected_airport.is_some(), |ui| {
        if let Some(airport) = &selected_airport {
            let text_color = Color32::from_rgba_unmultiplied(200, 200, 200, 255);
            ui.label(
                RichText::new(format!("\t{}", &airport.name))
                    .color(text_color)
                    .heading(),
            );
            ui.separator();

            ui.label(RichText::new(format!("\n\n\tIdent:\t{}", &airport.ident)).color(text_color));
            ui.label(
                RichText::new(format!("\tType:\t{}", &airport.airport_type)).color(text_color),
            );

            ui.label(
                RichText::new(format!(
                    "\n\tPosition:\t({}, {})",
                    &airport.position.lat(),
                    &airport.position.lon()
                ))
                .color(text_color),
            );
            ui.label(
                RichText::new(format!(
                    "\tElevation (ft):\t{}",
                    &airport.elevation_ft.unwrap_or(-999)
                ))
                .color(text_color),
            );

            ui.label(
                RichText::new(format!("\tContinent:\t{}", &airport.continent)).color(text_color),
            );

            ui.label(
                RichText::new(format!("\tCountry (ISO):\t{}", &airport.iso_country))
                    .color(text_color),
            );
        }
    });
}
