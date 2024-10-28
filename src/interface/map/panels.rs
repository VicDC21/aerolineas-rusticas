//! Módulo para paneles de la interfaz.

use std::sync::Arc;

use eframe::egui::{Button, Color32, Context, Frame, Margin, RichText, ScrollArea, SidePanel, Ui};

use crate::data::{airports::Airport, flights::Flight};

/// Muestra por un panel lateral los detalles del aeropuerto actualmente seleccionado.
pub fn cur_airport_info(ctx: &Context, cur_airport: &Option<Airport>, flights: Arc<Vec<Flight>>) {
    let panel_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(66, 66, 66, 200),
        inner_margin: Margin::ZERO,
        ..Default::default()
    };
    let info_panel = SidePanel::left("cur_airport_info")
        .resizable(false)
        .exact_width(ctx.screen_rect().width() / 3.0)
        .frame(panel_frame);
    info_panel.show_animated(ctx, cur_airport.is_some(), |ui| {
        show_airport_info(ui, cur_airport);

        ui.separator();

        let button = Button::new(RichText::new("Mostrar Vuelos").heading());
        if ui.add(button).clicked() {
            println!("Mostrando vuelos...");
            ScrollArea::vertical()
                .max_height(50.0)
                .show(ui, |scroll_ui| {
                    for flight in flights.iter() {
                        let potential_date = match flight.get_date() {
                            None => "".to_string(),
                            Some(date) => date.to_string(),
                        };
                        let info = format!(
                            "Id: {}\nOrigen: {}\nDestino: {}\nFecha: {}\n\n",
                            flight.id, flight.orig, flight.dest, potential_date,
                        );
                        scroll_ui.label(RichText::new(info).italics());
                    }
                });
        }
    });
}

/// Muestra por un panel lateral los detalles del aeropuerto extra.
pub fn extra_airport_info(
    ctx: &Context,
    selected_airport: &Option<Airport>,
    extra_airport: &Option<Airport>,
) {
    let panel_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(60, 60, 60, 200),
        inner_margin: Margin::ZERO,
        ..Default::default()
    };
    let info_panel = SidePanel::right("extra_airport_info")
        .resizable(false)
        .exact_width(ctx.screen_rect().width() / 3.0)
        .frame(panel_frame);
    info_panel.show_animated(ctx, extra_airport.is_some(), |ui| {
        show_airport_info(ui, extra_airport);
        ui.separator();

        if let Some(ex_airport) = extra_airport {
            if let Some(cur_airport) = selected_airport {
                let button = Button::new(RichText::new("Añadir Vuelo").heading());
                if ui.add_enabled(selected_airport.is_some(), button).clicked() {
                    println!(
                        "Agregando vuelo desde '{}' hasta '{}'...",
                        &cur_airport.name, &ex_airport.name
                    );
                }
            }
        }
    });
}

// Muestra la info de un aeropuerto.
fn show_airport_info(ui: &mut Ui, airport: &Option<Airport>) {
    if let Some(airport) = &airport {
        let text_color = Color32::from_rgba_unmultiplied(200, 200, 200, 255);
        ui.label(
            RichText::new(format!("\t{}", &airport.name))
                .color(text_color)
                .heading(),
        );
        ui.separator();

        ui.label(RichText::new(format!("\n\n\tIdent:\t{}", &airport.ident)).color(text_color));
        ui.label(RichText::new(format!("\tType:\t{}", &airport.airport_type)).color(text_color));

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

        ui.label(RichText::new(format!("\tContinent:\t{}", &airport.continent)).color(text_color));

        ui.label(
            RichText::new(format!("\tCountry (ISO):\t{}", &airport.iso_country)).color(text_color),
        );
    }
}
