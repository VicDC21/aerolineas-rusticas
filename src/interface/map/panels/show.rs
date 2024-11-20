//! Módulo para paneles de la interfaz.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use eframe::egui::{
    Button, Color32, Context, Frame, Margin, Response, RichText, ScrollArea, Separator, SidePanel,
    Ui,
};

use crate::{
    client::cli::Client,
    data::{
        airports::airp::Airport,
        flights::{departing::DepartingFlight, incoming::IncomingFlight, traits::Flight},
    },
    interface::map::panels::crud::{delete_flight_by_id, insert_flight},
    protocol::aliases::types::{Int, Long},
};

/// IDs de aeropuertos a borrar después.
pub type DeleteQueue = HashSet<Int>;

/// Muestra por un panel lateral los detalles del aeropuerto actualmente seleccionado.
pub fn cur_airport_info(
    client: Arc<Mutex<Client>>,
    ctx: &Context,
    cur_airport: &Option<Airport>,
    incoming_flights: Arc<Vec<IncomingFlight>>,
    show_incoming: &bool,
    departing_flights: Arc<Vec<DepartingFlight>>,
    show_departing: &bool,
) -> (bool, bool) {
    let mut must_show_incoming = *show_incoming;
    let mut must_show_departing = *show_departing;
    let mut delete_queue = DeleteQueue::new();
    let panel_width = ctx.screen_rect().width() / 3.0;

    let panel_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(66, 66, 66, 200),
        inner_margin: Margin::ZERO,
        ..Default::default()
    };
    let info_panel = SidePanel::left("cur_airport_info")
        .resizable(false)
        .exact_width(panel_width)
        .frame(panel_frame);
    info_panel.show_animated(ctx, cur_airport.is_some(), |ui| {
        ui.style_mut().spacing.indent = 3.0; // Para que no se pegue a los bordes
        show_airport_info(ui, cur_airport);
        ui.separator();

        let mut buttons = HashMap::<&str, Response>::new();
        let can_show = |show| if show { "Ocultar" } else { "Mostrar" };
        ui.horizontal(|horizontal_ui| {
            buttons.insert(
                "incoming",
                horizontal_ui.button(
                    RichText::new(format!("{} Vuelos Entrantes", can_show(must_show_incoming)))
                        .heading(),
                ),
            );
            buttons.insert(
                "departing",
                horizontal_ui.button(
                    RichText::new(format!(
                        "{} Vuelos Salientes",
                        can_show(must_show_departing)
                    ))
                    .heading(),
                ),
            );
        });

        must_show_incoming = show_incoming_flights(
            ctx,
            ui,
            &buttons["incoming"],
            show_incoming,
            incoming_flights,
            &mut delete_queue,
        );
        must_show_departing = show_departing_flights(
            ctx,
            ui,
            &buttons["departing"],
            show_departing,
            departing_flights,
            &mut delete_queue,
        );
    });

    if !delete_queue.is_empty() {
        for flight_id in delete_queue {
            let _ = delete_flight_by_id(Arc::clone(&client), flight_id);
        }
    }

    (must_show_incoming, must_show_departing)
}

/// Muestra por un panel lateral los detalles del aeropuerto extra.
pub fn extra_airport_info(
    client: Arc<Mutex<Client>>,
    ctx: &Context,
    selected_airport: &Option<Airport>,
    extra_airport: &Option<Airport>,
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
            RichText::new(format!(
                "\tCountry:\t{} ({})",
                &airport.country.name, &airport.country.code
            ))
            .color(text_color),
        );
    }
}

/// Muestra lso vuelos entrantes.
fn show_incoming_flights(
    ctx: &Context,
    ui: &mut Ui,
    incoming_button_response: &Response,
    show_incoming: &bool,
    incoming_flights: Arc<Vec<IncomingFlight>>,
    delete_queue: &mut DeleteQueue,
) -> bool {
    let mut must_show_incoming = *show_incoming;
    if must_show_incoming {
        ScrollArea::vertical()
            .max_height(100.0)
            .max_width(ctx.screen_rect().width() / 3.5)
            .animated(true)
            .id_salt("incoming_scroll")
            .show(ui, |scroll_ui| {
                if incoming_flights.is_empty() {
                    show_loading_spinner(scroll_ui, "Cargando vuelos entrantes...".to_string());
                }

                for incoming_flight in incoming_flights.iter() {
                    let potential_date = match incoming_flight.get_date() {
                        None => "".to_string(),
                        Some(date) => date.to_string(),
                    };
                    scroll_ui.label(
                        RichText::new(format!("\tId: {}\n", incoming_flight.id))
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                    scroll_ui.label(
                        RichText::new(format!("\tDestino: {}", incoming_flight.dest))
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                    scroll_ui.label(
                        RichText::new(format!("\tETA: {}", potential_date))
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                    scroll_ui.label(
                        RichText::new(format!(
                            "\tPosición (lat, lon): {:?}",
                            (incoming_flight.pos.lat(), incoming_flight.pos.lon())
                        ))
                        .italics()
                        .color(Color32::from_rgb(255, 255, 255)),
                    );
                    scroll_ui.label(
                        RichText::new(format!("\testado: {}", incoming_flight.state))
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                    if scroll_ui.button("Borrar").clicked() {
                        delete_queue.insert(incoming_flight.id);
                    }
                    scroll_ui.add(Separator::default().shrink(30.0));
                }
            });
    }
    if incoming_button_response.clicked() {
        if *show_incoming {
            println!("Ocultando vuelos entrantes...");
            must_show_incoming = false;
        } else {
            println!("Mostrando vuelos entrantes...");
            must_show_incoming = true;
        }
    }
    must_show_incoming
}

/// Muestra los vuelos salientes.
fn show_departing_flights(
    ctx: &Context,
    ui: &mut Ui,
    departing_button_response: &Response,
    show_departing: &bool,
    departing_flights: Arc<Vec<DepartingFlight>>,
    delete_queue: &mut DeleteQueue,
) -> bool {
    let mut must_show_departing = *show_departing;
    if must_show_departing {
        ScrollArea::vertical()
            .max_height(100.0)
            .max_width(ctx.screen_rect().width() / 3.5)
            .animated(true)
            .id_salt("departing_scroll")
            .show(ui, |scroll_ui| {
                if departing_flights.is_empty() {
                    show_loading_spinner(scroll_ui, "Cargando vuelos salientes...".to_string());
                }

                for departing_flight in departing_flights.iter() {
                    let potential_date = match departing_flight.get_date() {
                        None => "".to_string(),
                        Some(date) => date.to_string(),
                    };
                    scroll_ui.label(
                        RichText::new(format!("\tId: {}\n", departing_flight.id))
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                    scroll_ui.label(
                        RichText::new(format!("\tOrigen: {}", departing_flight.orig))
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                    scroll_ui.label(
                        RichText::new(format!("\tDespegue: {}", potential_date))
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                    scroll_ui.label(
                        RichText::new(format!(
                            "\tPosición (lat, lon): {:?}",
                            (departing_flight.pos.lat(), departing_flight.pos.lon())
                        ))
                        .italics()
                        .color(Color32::from_rgb(255, 255, 255)),
                    );
                    scroll_ui.label(
                        RichText::new(format!("\testado: {}", departing_flight.state))
                            .italics()
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                    if scroll_ui.button("Borrar").clicked() {
                        delete_queue.insert(departing_flight.id);
                    }
                    scroll_ui.add(Separator::default().shrink(30.0));
                }
            });
    }
    if departing_button_response.clicked() {
        if *show_departing {
            println!("Ocultando vuelos salientes...");
            must_show_departing = false;
        } else {
            println!("Mostrando vuelos salientes...");
            must_show_departing = true;
        }
    }
    must_show_departing
}

/// Muestra un mensaje de carga.
fn show_loading_spinner(ui: &mut Ui, msg: String) -> Response {
    ui.vertical_centered(|loading_ui| {
        loading_ui.label(
            RichText::new(msg)
                .italics()
                .color(Color32::from_rgb(255, 255, 255)),
        );
        loading_ui.spinner();
    })
    .response
}
