//! Módulo para funciones que crean ventanas en el mapa.
//!
//! Al ser "ventanas" flotantes, se pueden mostrar por encima del mapa.

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, Timelike};
use eframe::egui::{Align2, ComboBox, Context, RichText, Ui, Window};
use egui_extras::DatePickerButton;
use walkers::MapMemory;

/// Zoom simple.
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

    Window::new("Clock Selector")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [220., -10.])
        .show(ctx, |ui| {
            ui.collapsing(
                RichText::new(format!("{:0>2}:{:0>2}:{:0>2}", &hour, &minute, &second)),
                |ui| {
                    ComboBox::from_label("Hora")
                        .selected_text(format!("{}", hour))
                        .show_ui(ui, |ui| {
                            for h in 0..24 {
                                ui.selectable_value(&mut hour, h, format!("{:0>2}", h));
                            }
                        });
                    ComboBox::from_label("Minutos")
                        .selected_text(format!("{}", minute))
                        .show_ui(ui, |ui| {
                            for m in 0..60 {
                                ui.selectable_value(&mut minute, m, format!("{:0>2}", m));
                            }
                        });
                    ComboBox::from_label("Segundos")
                        .selected_text(format!("{}", second))
                        .show_ui(ui, |ui| {
                            for s in 0..60 {
                                ui.selectable_value(&mut second, s, format!("{:0>2}", s));
                            }
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
