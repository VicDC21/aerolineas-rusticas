//! Módulo para funciones que crean ventanas en el mapa.
//!
//! Al ser "ventanas" flotantes, se pueden mostrar por encima del mapa.

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, Timelike};
use eframe::egui::{Align2, ComboBox, Context, DragValue, Image, RichText, Ui, Window};
use egui_extras::DatePickerButton;
use walkers::{sources::Attribution, MapMemory};

use crate::interface::map::providers::Provider;

/// Crea un botón con un link a los creadores del proveedor.
pub fn acknowledge(ui: &Ui, attribution: Attribution) {
    Window::new("Acknowledge")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_TOP, [10., 10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if let Some(logo) = attribution.logo_light {
                    ui.add(Image::new(logo).max_height(30.0).max_width(80.0));
                }
                ui.hyperlink_to(attribution.text, attribution.url);
            });
        });
}

/// Crea un seleccionador de proveedor.
pub fn controls(
    ui: &Ui,
    selected_provider: &mut Provider,
    possible_providers: &mut dyn Iterator<Item = &Provider>,
    // image: &mut ImagesPluginData,
) {
    Window::new("Satellite")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 10.])
        .fixed_size([150., 150.])
        .show(ui.ctx(), |ui| {
            ui.collapsing("Map", |ui| {
                ComboBox::from_label("Tile Provider")
                    .selected_text(format!("{:?}", selected_provider))
                    .show_ui(ui, |ui| {
                        for p in possible_providers {
                            ui.selectable_value(selected_provider, *p, format!("{:?}", p));
                        }
                    });
            });
        });
}

/// Simple GUI to zoom in and out.
pub fn zoom(ui: &Ui, map_memory: &mut MapMemory) {
    Window::new("Zoom Buttons")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [10., -10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("➕").heading()).clicked() {
                    let _ = map_memory.zoom_in();
                }

                if ui.button(RichText::new("➖").heading()).clicked() {
                    let _ = map_memory.zoom_out();
                }
            });
        });
}

/// Cuando el foco se mueve del origen de coordenadas, aparece este botón para traerte de vuelta.
pub fn go_to_my_position(ui: &Ui, map_memory: &mut MapMemory) {
    if map_memory.detached().is_some() {
        Window::new("Follow Pos")
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::RIGHT_BOTTOM, [-10., -10.])
            .show(ui.ctx(), |ui| {
                if ui.button(RichText::new("📌").heading()).clicked() {
                    map_memory.follow_my_position();
                }
            });
    }
}

/// Seleccionar la fecha actual.
pub fn date_selector(ctx: &Context, datetime: &mut DateTime<Local>) -> Option<DateTime<Local>> {
    let mut date = datetime.date_naive();
    Window::new("Date Selector")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [100., -10.])
        .show(ctx, |ui| {
            ui.add(DatePickerButton::new(&mut date).id_salt("date_selector"));
        });

    let local = datetime.naive_local();
    NaiveDateTime::new(date, local.time())
        .and_local_timezone(Local)
        .single()
}

/// Seleccionar la hora actual.
pub fn clock_selector(ctx: &Context, datetime: &mut DateTime<Local>) -> Option<DateTime<Local>> {
    let mut hour = datetime.hour();
    let mut minute = datetime.minute();
    let mut second = datetime.second();
    let slider_spd = 0.5;

    Window::new("Clock Selector")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [220., -10.])
        .show(ctx, |ui| {
            ui.collapsing(
                RichText::new(format!("{:0<2}:{:0<2}:{:0<2}", &hour, &minute, &second)),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hora:").heading());
                        ui.add(DragValue::new(&mut hour).range(0..=23).speed(slider_spd));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Minutos:").heading());
                        ui.add(DragValue::new(&mut minute).range(0..=59).speed(slider_spd));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Segundos:").heading());
                        ui.add(DragValue::new(&mut second).range(0..=59).speed(slider_spd));
                    });
                },
            );
        });
    if let Some(time) = NaiveTime::from_hms_opt(hour, minute, second) {
        datetime.with_time(time).single()
    } else {
        None
    }
}
