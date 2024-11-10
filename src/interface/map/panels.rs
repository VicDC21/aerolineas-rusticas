//! Módulo para paneles de la interfaz.

use std::sync::{Arc, Mutex};

use eframe::egui::{Button, Color32, Context, Frame, Margin, RichText, ScrollArea, SidePanel, Ui};

use crate::{
    client::{cli::Client, protocol_result::ProtocolResult},
    data::{airports::Airport, flight_states::FlightState, flights::Flight},
    protocol::{
        aliases::{
            results::Result,
            types::{Int, Long},
        },
        errors::error::Error,
    },
};

/// Muestra por un panel lateral los detalles del aeropuerto actualmente seleccionado.
pub fn cur_airport_info(
    ctx: &Context,
    cur_airport: &Option<Airport>,
    flights: Arc<Vec<Flight>>,
    show_details: &bool,
) -> bool {
    let mut must_show = *show_details;

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
        if *show_details {
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
                        scroll_ui.label(
                            RichText::new(info)
                                .italics()
                                .color(Color32::from_rgb(255, 255, 255)),
                        );
                    }
                });
        }
        if ui.add(button).clicked() {
            if *show_details {
                println!("Mostrando vuelos...");
                must_show = false;
            } else {
                println!("Ocultando vuelos...");
                must_show = true;
            }
        }
    });
    must_show
}

/// Muestra por un panel lateral los detalles del aeropuerto extra.
pub fn extra_airport_info(
    ctx: &Context,
    selected_airport: &Option<Airport>,
    extra_airport: &Option<Airport>,
    client: Client,
    timestamp: Long,
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
                if ui.add_enabled(selected_airport.is_some() && extra_airport.is_some(), button).clicked() {
                    if let Err(err) = insert_flight(client, timestamp, cur_airport, ex_airport) {
                        println!("Ocurrió un error tratando de agregar un vuelo desde '{}' hasta '{}'\n\n{}",
                                 &cur_airport.name, &ex_airport.name, err);
                    }
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

/// Inserta un nuevo vuelo.
fn insert_flight(
    client: Client,
    timestamp: Long,
    cur_airport: &Airport,
    ex_airport: &Airport,
) -> Result<()> {
    let flight_id = cur_airport.id + ex_airport.id + timestamp as usize;

    let incoming_client = Arc::new(Mutex::new(client));
    // TODO: calcular el tiempo estimado por distancia en vez de asumir que el vuelo es instantáneo.
    let incoming_query = format!(
        "INSERT INTO vuelos_entrantes (id, dest, llegada, pos_lat, pos_lon, estado) VALUES ({}, '{}', {}, {}, {}, '{}');",
        flight_id as Int, ex_airport.ident, timestamp, cur_airport.position.lat(), cur_airport.position.lon(), FlightState::InCourse
    );

    let departing_client = Arc::clone(&incoming_client);
    let departing_query = format!(
        "INSERT INTO vuelos_salientes (id, orig, salida, pos_lat, pos_lon, estado) VALUES ({}, '{}', {}, {}, {}, '{}');",
        flight_id as Int, cur_airport.ident, timestamp, cur_airport.position.lat(), cur_airport.position.lon(), FlightState::InCourse
    );

    send_insert_query(incoming_client, incoming_query.as_str())?;
    send_insert_query(departing_client, departing_query.as_str())?;

    Ok(())
}

/// Manda una _query_ para insertar un tipo de vuelo.
fn send_insert_query(client_lock: Arc<Mutex<Client>>, query: &str) -> Result<()> {
    let mut client = match client_lock.lock() {
        Ok(cli) => cli,
        Err(poison_err) => {
            return Err(Error::ServerError(format!(
                "Error de lock envenenado tratando de leer un cliente:\n\n{}",
                poison_err
            )))
        }
    };

    let mut tcp_stream = client.connect()?;
    let protocol_result = client.send_query(query, &mut tcp_stream)?;

    if let ProtocolResult::QueryError(err) = protocol_result {
        println!("{}", err);
    }

    Ok(())
}
