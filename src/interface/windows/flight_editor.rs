//! Módulo para el editor de detalles de un vuelo.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local};
use eframe::egui::{Align2, Color32, ComboBox, Frame, Key, RichText, Ui, Window};

use crate::{
    client::cli::Client,
    data::{
        flights::{flight::Flight, states::FlightState, types::FlightType},
        tracking::live_flight_data::LiveFlightData,
        traits::PrettyShow,
    },
    interface::utils::send_client_query,
};

/// Editor para modificar detalles de un vuelo en curso y sus datos en vivo.
pub struct FlightEditorWindow {
    /// El vuelo guardado.
    pub held_flight: Option<Flight>,

    /// El aeropuerto de origen.
    pub orig: String,

    /// El aeropuerto de destino.
    pub dest: String,

    /// La fecha de arribo o salida.
    pub date: Option<DateTime<Local>>,

    /// El estado del vuelo.
    pub state: FlightState,
}

impl FlightEditorWindow {
    /// Crea una nueva instancia.
    pub fn new(
        held_flight: Option<Flight>,
        orig: String,
        dest: String,
        date: Option<DateTime<Local>>,
        state: FlightState,
    ) -> Self {
        Self {
            held_flight,
            orig,
            dest,
            date,
            state,
        }
    }

    /// Muestra la ventana del editor.
    pub fn show(
        &mut self,
        ui: &Ui,
        client: Arc<Mutex<Client>>,
        local_date: DateTime<Local>,
        live_data: Option<LiveFlightData>,
    ) -> bool {
        let ctx = ui.ctx();
        let mut keep_open = true;

        if let Some(flight) = &self.held_flight {
            let text_color = Color32::from_rgb(200, 200, 200);
            let table = match flight.flight_type {
                FlightType::Incoming => "vuelos_entrantes",
                FlightType::Departing => "vuelos_salientes",
            };

            Window::new(format!("Flight {}", flight.id))
                .collapsible(false)
                .resizable(false)
                .fixed_size([
                    ctx.screen_rect().height() * 0.9,
                    ctx.screen_rect().width() * 0.6,
                ])
                .title_bar(true)
                .fade_in(true)
                .fade_out(true)
                .frame(Frame::none().fill(Color32::from_rgb(60, 60, 60)))
                .open(&mut keep_open)
                .anchor(Align2::CENTER_TOP, [0., 200.])
                .show(ctx, |win_ui| {
                    win_ui.horizontal(|hor_ui| {
                        hor_ui.label(
                            RichText::new(format!("{:<15}{:>40}", "ID:", flight.id,))
                                .monospace()
                                .color(text_color),
                        );
                    });
                    win_ui.horizontal(|hor_ui| {
                        hor_ui.label(
                            RichText::new(format!("{:<15}", "Origin:"))
                                .monospace()
                                .color(text_color),
                        );
                        if hor_ui.text_edit_singleline(&mut self.orig).lost_focus()
                            && hor_ui.input(|i| i.key_pressed(Key::Enter))
                        {
                            if let Err(err) = send_client_query(
                                Arc::clone(&client),
                                format!(
                                    "UPDATE {} SET orig = '{}' WHERE id = {};",
                                    table, self.orig, flight.id,
                                )
                                .as_str(),
                            ) {
                                println!(
                                    "Ocurrió un error actualizando el origen del vuelo:\n\n{}",
                                    err
                                );
                            }
                        }
                    });
                    win_ui.horizontal(|hor_ui| {
                        hor_ui.label(
                            RichText::new(format!("{:<15}", "Destination:"))
                                .monospace()
                                .color(text_color),
                        );
                        if hor_ui.text_edit_singleline(&mut self.dest).lost_focus()
                            && hor_ui.input(|i| i.key_pressed(Key::Enter))
                        {
                            if let Err(err) = send_client_query(
                                Arc::clone(&client),
                                format!(
                                    "UPDATE {} SET dest = '{}' WHERE id = {};",
                                    table, self.dest, flight.id,
                                )
                                .as_str(),
                            ) {
                                println!(
                                    "Ocurrió un error actualizando el destino del vuelo:\n\n{}",
                                    err
                                );
                            }
                        }
                    });
                    win_ui.horizontal(|hor_ui| {
                        hor_ui.label(
                            RichText::new(format!("{:<15}", "Date:"))
                                .monospace()
                                .color(text_color),
                        );
                        let date_str = match self.date {
                            Some(d) => d.to_string(),
                            None => "(Not Available)".to_string(),
                        };
                        hor_ui.label(
                            RichText::new(format!("{:>20}", date_str))
                                .monospace()
                                .color(text_color),
                        );
                        if hor_ui
                            .button(RichText::new("Set").color(text_color))
                            .clicked()
                        {
                            let timestamp_col = match flight.flight_type {
                                FlightType::Incoming => "llegada",
                                FlightType::Departing => "salida",
                            };
                            if let Err(err) = send_client_query(
                                Arc::clone(&client),
                                format!(
                                    "UPDATE {} SET {} = {} WHERE id = {};",
                                    table,
                                    timestamp_col,
                                    local_date.timestamp(),
                                    flight.id,
                                )
                                .as_str(),
                            ) {
                                println!(
                                    "Ocurrió un error actualizando la fecha del vuelo:\n\n{}",
                                    err
                                );
                            }
                            self.date = Some(local_date);
                        }
                    });
                    win_ui.horizontal(|hor_ui| {
                        hor_ui.label(
                            RichText::new(format!("{:<15}", "State:"))
                                .monospace()
                                .color(text_color),
                        );
                        ComboBox::from_id_salt(format!("flight_{}_state", flight.id))
                            .selected_text(self.state.pretty_name())
                            .show_ui(hor_ui, |combo_ui| {
                                combo_ui.selectable_value(
                                    &mut self.state,
                                    FlightState::InCourse,
                                    FlightState::InCourse.pretty_name(),
                                );
                                combo_ui.selectable_value(
                                    &mut self.state,
                                    FlightState::Delayed,
                                    FlightState::Delayed.pretty_name(),
                                );
                                combo_ui.selectable_value(
                                    &mut self.state,
                                    FlightState::Canceled,
                                    FlightState::Canceled.pretty_name(),
                                );
                                combo_ui.selectable_value(
                                    &mut self.state,
                                    FlightState::Finished,
                                    FlightState::Finished.pretty_name(),
                                );
                                combo_ui.selectable_value(
                                    &mut self.state,
                                    FlightState::Preparing,
                                    FlightState::Preparing.pretty_name(),
                                );
                            });
                        if self.state != flight.state {
                            if let Err(err) = send_client_query(
                                Arc::clone(&client),
                                format!(
                                    "UPDATE {} SET state = '{}' WHERE id = {};",
                                    table, self.state, flight.id,
                                )
                                .as_str(),
                            ) {
                                println!(
                                    "Ocurrió un error actualizando el estado del vuelo:\n\n{}",
                                    err
                                );
                            }
                        }
                    });
                    win_ui.horizontal(|hor_ui| {
                        hor_ui.label(RichText::new(format!(
                            "{:<10}{:>20}",
                            "Type:",
                            flight.flight_type.pretty_name(),
                        )));
                    });

                    // -- datos de tracking --

                    if let Some(tracking_data) = live_data {
                        win_ui.add_space(50.0);

                        win_ui.label(RichText::new(format!(
                            "{:<15}{:>25}",
                            "Average Speed:",
                            tracking_data.avg_spd(),
                        )));
                        win_ui.label(RichText::new(format!(
                            "{:<15}{:>25}",
                            "Fuel Level:", tracking_data.fuel,
                        )));
                        win_ui.label(RichText::new(format!(
                            "{:<15}{:>25}",
                            "Position:",
                            format!("({:.4}, {:.4})", tracking_data.pos.0, tracking_data.pos.1),
                        )));
                        win_ui.label(RichText::new(format!(
                            "{:<15}{:>25}",
                            "Altitude (ft):", tracking_data.altitude_ft,
                        )));
                        win_ui.label(RichText::new(format!(
                            "{:<15}{:>25}",
                            "Time Elapsed:",
                            format!("{:.5}", tracking_data.elapsed.as_secs()),
                        )));
                    }
                });
        }

        keep_open
    }
}

impl From<&Flight> for FlightEditorWindow {
    fn from(flight: &Flight) -> Self {
        Self::new(
            Some(flight.clone()),
            flight.orig.to_string(),
            flight.dest.to_string(),
            flight.get_date(),
            flight.state.clone(),
        )
    }
}
