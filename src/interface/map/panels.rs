//! Módulo para paneles de la interfaz.

use std::sync::{Arc, Mutex};

use eframe::egui::{Button, Color32, Context, Frame, Margin, RichText, ScrollArea, SidePanel, Ui};

use crate::{
    client::{cli::Client, protocol_result::ProtocolResult},
    data::{
        airports::Airport,
        flights::{
            departing::DepartingFlight, incoming::IncomingFlight, states::FlightState,
            traits::Flight,
        },
    },
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
    incoming_flights: Arc<Vec<IncomingFlight>>,
    show_incoming: &bool,
    departing_flights: Arc<Vec<DepartingFlight>>,
    show_departing: &bool,
) -> (bool, bool) {
    let mut must_show_incoming = *show_incoming;
    let mut must_show_departing = *show_departing;

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
        must_show_incoming = show_incoming_flights(ctx, ui, show_incoming, incoming_flights);
        ui.separator();
        must_show_departing = show_departing_flights(ctx, ui, show_departing, departing_flights);
    });
    (must_show_incoming, must_show_departing)
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

/// Muestra lso vuelos entrantes.
fn show_incoming_flights(
    ctx: &Context,
    ui: &mut Ui,
    show_incoming: &bool,
    incoming_flights: Arc<Vec<IncomingFlight>>,
) -> bool {
    let mut must_show_incoming = *show_incoming;
    let incoming_button = Button::new(RichText::new("Mostrar Vuelos Entrantes").heading());
    if must_show_incoming {
        ScrollArea::vertical()
            .max_height(100.0)
            .max_width(ctx.screen_rect().width() / 4.0)
            .show(ui, |scroll_ui| {
                for incoming_flight in incoming_flights.iter() {
                    let potential_date = match incoming_flight.get_date() {
                        None => "".to_string(),
                        Some(date) => date.to_string(),
                    };
                    let info = format!(
                        "Id: {}\n\nDestino: {}\nETA: {}\nPosición (lat, lon): {:?}, estado: {}\n",
                        incoming_flight.id,
                        incoming_flight.dest,
                        potential_date,
                        (incoming_flight.pos.lat(), incoming_flight.pos.lon()),
                        incoming_flight.state
                    );
                    scroll_ui.label(
                        RichText::new(info)
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                }
            });
    }
    if ui.add(incoming_button).clicked() {
        if *show_incoming {
            println!("Mostrando vuelos entrantes...");
            must_show_incoming = false;
        } else {
            println!("Ocultando vuelos entrantes...");
            must_show_incoming = true;
        }
    }
    must_show_incoming
}

/// Muestra los vuelos salientes.
fn show_departing_flights(
    ctx: &Context,
    ui: &mut Ui,
    show_departing: &bool,
    departing_flights: Arc<Vec<DepartingFlight>>,
) -> bool {
    let mut must_show_departing = *show_departing;
    let departing_button = Button::new(RichText::new("Mostrar Vuelos Salientes").heading());
    if must_show_departing {
        ScrollArea::vertical()
                .max_height(100.0)
                .max_width(ctx.screen_rect().width() / 4.0)
                .show(ui, |scroll_ui| {
                    for departing_flight in departing_flights.iter() {
                        let potential_date = match departing_flight.get_date() {
                            None => "".to_string(),
                            Some(date) => date.to_string(),
                        };
                        let info = format!(
                            "Id: {}\n\nOrigen: {}\nDespegue: {}\nPosición (lat, lon): {:?}, estado: {}\n",
                            departing_flight.id, departing_flight.orig, potential_date, (departing_flight.pos.lat(), departing_flight.pos.lon()), departing_flight.state
                        );
                        scroll_ui.label(
                            RichText::new(info)
                                .italics()
                                .color(Color32::from_rgb(255, 255, 255)),
                        );
                    }
                });
    }
    if ui.add(departing_button).clicked() {
        if *show_departing {
            println!("Mostrando vuelos salientes...");
            must_show_departing = false;
        } else {
            println!("Ocultando vuelos salientes...");
            must_show_departing = true;
        }
    }
    must_show_departing
}
