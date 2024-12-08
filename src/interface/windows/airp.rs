//! Módulo para ventanas de widgets de aeropuertos.

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, Timelike};
use eframe::egui::{Align2, ComboBox, ProgressBar, RichText, Ui, Window};
use egui_extras::DatePickerButton;

use crate::{client::conn_holder::ConnectionHolder, data::login_info::LoginInfo};

/// Seleccionar la fecha actual.
pub fn date_selector(ui: &Ui, datetime: &mut DateTime<Local>) -> Option<DateTime<Local>> {
    let mut date = datetime.date_naive();
    Window::new("Date Selector")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [100., -10.])
        .show(ui.ctx(), |ui| {
            ui.add(DatePickerButton::new(&mut date).id_salt("date_selector"));
        });

    let local = datetime.naive_local();
    NaiveDateTime::new(date, local.time())
        .and_local_timezone(Local)
        .single()
}

/// Seleccionar la hora actual.
pub fn clock_selector(ui: &Ui, datetime: &mut DateTime<Local>) -> Option<DateTime<Local>> {
    let mut hour = datetime.hour();
    let mut minute = datetime.minute();
    let mut second = datetime.second();

    Window::new("Clock Selector")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [220., -10.])
        .show(ui.ctx(), |ui| {
            ui.collapsing(
                RichText::new(format!("{:0>2}:{:0>2}:{:0>2}", &hour, &minute, &second)),
                |ui| {
                    ComboBox::from_label("Hour")
                        .selected_text(format!("{}", hour))
                        .show_ui(ui, |ui| {
                            for h in 0..24 {
                                ui.selectable_value(&mut hour, h, format!("{:0>2}", h));
                            }
                        });
                    ComboBox::from_label("Minutes")
                        .selected_text(format!("{}", minute))
                        .show_ui(ui, |ui| {
                            for m in 0..60 {
                                ui.selectable_value(&mut minute, m, format!("{:0>2}", m));
                            }
                        });
                    ComboBox::from_label("Seconds")
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

/// Crea una barra de progreso que indica la cantidad de aeropuertos por cargar.
pub fn airports_progress(ui: &Ui, start: usize, end: usize) {
    let ctx = ui.ctx();
    Window::new("Airports Loading Progress")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::CENTER_TOP, [0., 25.])
        .show(ctx, |ui| {
            let progress = start as f32 / end as f32;
            let progress_bar = ProgressBar::new(progress)
                .desired_width(ctx.screen_rect().width() / 5.)
                .animate(true)
                .text(format!(
                    "Cargando aeropuertos: {} / {} ({:.2}%)",
                    start,
                    end,
                    progress * 100.
                ));
            ui.add(progress_bar);
        });
}

/// Crea una ventanita de logueo.
pub fn login_window(ui: &Ui, conn: &mut ConnectionHolder, login_info: &mut LoginInfo) {
    let ctx = ui.ctx();

    Window::new("Login")
        .collapsible(true)
        .resizable(false)
        .anchor(Align2::RIGHT_TOP, [-10., -10.])
        .show(ctx, |win_ui| {
            win_ui.horizontal(|hor_ui| {
                hor_ui.label(RichText::new(format!("{:<15}", "User:")).heading());
                hor_ui.text_edit_singleline(&mut login_info.user);
            });
            win_ui.horizontal(|hor_ui| {
                hor_ui.label(RichText::new(format!("{:<15}", "Password:")).heading());
                hor_ui.text_edit_singleline(&mut login_info.pass);
            });
            if win_ui.button(RichText::new("LOGIN").heading()).clicked() {
                let _ = conn.login(login_info);
            }
        });
}
