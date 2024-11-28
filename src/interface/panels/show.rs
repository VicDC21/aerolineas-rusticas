//! Módulo para paneles de la interfaz.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use eframe::egui::{
    Button, Color32, FontId, Frame, Margin, Response, RichText, ScrollArea, Separator, SidePanel,
    Ui,
};

use crate::{
    client::cli::Client,
    data::{
        airports::airp::Airport,
        flights::{flight::Flight, types::FlightType},
        traits::PrettyShow,
    },
    interface::{
        data::widget_details::WidgetDetails,
        panels::crud::{delete_flight_by_id, insert_flight},
        windows::flight_editor::FlightEditorWindow,
    },
    protocol::aliases::types::{Int, Long},
};

/// IDs de aeropuertos a borrar después.
pub type DeleteQueue = HashSet<Int>;

/// Muestra por un panel lateral los detalles del aeropuerto actualmente seleccionado.
pub fn cur_airport_info(
    client: Arc<Mutex<Client>>,
    ui: &Ui,
    cur_airport: &Option<Airport>,
    incoming_fl: (Arc<Vec<Flight>>, &bool),
    departing_fl: (Arc<Vec<Flight>>, &bool),
    widget_details: &mut WidgetDetails,
) -> (bool, bool) {
    let (incoming_flights, show_incoming) = incoming_fl;
    let (departing_flights, show_departing) = departing_fl;
    let mut must_show_incoming = *show_incoming;
    let mut must_show_departing = *show_departing;
    let mut delete_queue = DeleteQueue::new();
    let ctx = ui.ctx();
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

        ui.add_space(20.0);

        must_show_incoming = show_flights(
            ui,
            &buttons["incoming"],
            show_incoming,
            incoming_flights,
            &FlightType::Incoming,
            widget_details,
            &mut delete_queue,
        );
        must_show_departing = show_flights(
            ui,
            &buttons["departing"],
            show_departing,
            departing_flights,
            &FlightType::Departing,
            widget_details,
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
    ui: &Ui,
    selected_airport: &Option<Airport>,
    extra_airport: &Option<Airport>,
    timestamp: Long,
) {
    let panel_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(60, 60, 60, 200),
        inner_margin: Margin::ZERO,
        ..Default::default()
    };
    let ctx = ui.ctx();
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
        let font = FontId::monospace(15.0);
        let section_space = 10.0;

        ui.label(
            RichText::new(format!("\t{}", &airport.name))
                .color(text_color)
                .font(FontId::monospace(20.0))
                .heading(),
        );

        ui.separator();

        ui.label(
            RichText::new(format!("\t{:<20}{:>25}", "Ident:", &airport.ident))
                .color(text_color)
                .font(font.clone()),
        );
        ui.label(
            RichText::new(format!(
                "\t{:<20}{:>25}",
                "Type:",
                &airport.airport_type.pretty_name()
            ))
            .color(text_color)
            .font(font.clone()),
        );

        ui.add_space(section_space);

        ui.label(
            RichText::new(format!(
                "\t{:<20}{:>25}",
                "Position:",
                format!("({:.4}, {:.4})", &airport.position.0, &airport.position.1),
            ))
            .color(text_color)
            .font(font.clone()),
        );
        ui.label(
            RichText::new(format!(
                "\t{:<20}{:>25}",
                "Elevation (ft):",
                &airport.elevation_ft.unwrap_or(-999)
            ))
            .color(text_color)
            .font(font.clone()),
        );

        ui.add_space(section_space);

        ui.label(
            RichText::new(format!(
                "\t{:<20}{:>25}",
                "Continent:",
                format!(
                    "{} ({})",
                    &airport.continent.pretty_name(),
                    &airport.continent
                )
            ))
            .color(text_color)
            .font(font.clone()),
        );

        ui.label(
            RichText::new(format!(
                "\t{:<20}{:>25}",
                "Country:",
                format!("{} ({})", &airport.country.name, &airport.country.code),
            ))
            .color(text_color)
            .font(font.clone()),
        );
    }
}

/// Muestra los vuelos de un cierto tipo.
fn show_flights(
    ui: &mut Ui,
    button_response: &Response,
    do_show: &bool,
    flights: Arc<Vec<Flight>>,
    flight_type: &FlightType,
    widget_details: &mut WidgetDetails,
    delete_queue: &mut DeleteQueue,
) -> bool {
    let ctx = ui.ctx();
    let mut must_show = *do_show;
    if must_show {
        let text_color = Color32::from_rgb(255, 255, 255);
        let font = FontId::monospace(12.5);
        let interspace = 10.0;
        ScrollArea::vertical()
            .max_height(250.0)
            .max_width(ctx.screen_rect().width() / 3.5)
            .animated(true)
            .id_salt(format!("scroll_{}", flight_type))
            .show(ui, |scroll_ui| {
                if flights.is_empty() {
                    show_loading_spinner(scroll_ui, format!("Loading {} flights...", flight_type));
                }

                for flight in flights.iter() {
                    let potential_date = match flight.get_date() {
                        None => "".to_string(),
                        Some(date) => date.to_string(),
                    };
                    scroll_ui.label(
                        RichText::new(format!("\t{:<15}{:>30}", "Id:", flight.id))
                            .italics()
                            .color(text_color)
                            .font(font.clone()),
                    );
                    scroll_ui.label(
                        RichText::new(format!("\t{:<15}{:>30}", "Origin:", flight.orig))
                            .italics()
                            .color(text_color)
                            .font(font.clone()),
                    );
                    scroll_ui.label(
                        RichText::new(format!("\t{:<15}{:>30}", "Destination:", flight.dest))
                            .italics()
                            .color(text_color)
                            .font(font.clone()),
                    );
                    let tiemstamp_msg = match flight_type {
                        FlightType::Incoming => "Arrival:",
                        FlightType::Departing => "Take Off:",
                    };
                    scroll_ui.label(
                        RichText::new(format!("\t{:<15}{:>30}", tiemstamp_msg, potential_date))
                            .italics()
                            .color(text_color)
                            .font(font.clone()),
                    );
                    scroll_ui.label(
                        RichText::new(format!(
                            "\t{:<15}{:>30}",
                            "State:",
                            flight.state.pretty_name()
                        ))
                        .italics()
                        .color(text_color)
                        .font(font.clone()),
                    );
                    scroll_ui.horizontal(|hor_ui| {
                        hor_ui.add_space(30.0);
                        if hor_ui.button(RichText::new("Edit").heading()).clicked() {
                            widget_details.flight_editor = Some(FlightEditorWindow::from(flight));
                        }
                        if hor_ui.button(RichText::new("Delete").heading()).clicked() {
                            delete_queue.insert(flight.id);
                        }
                    });
                    scroll_ui.add_space(interspace);
                    scroll_ui.add(Separator::default().shrink(30.0));
                    scroll_ui.add_space(interspace);
                }
            });
    }
    if button_response.clicked() {
        if *do_show {
            println!("Ocultando vuelos salientes...");
            must_show = false;
        } else {
            println!("Mostrando vuelos salientes...");
            must_show = true;
        }
    }
    must_show
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
