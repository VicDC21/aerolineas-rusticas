//! Módulo para plugin que va actualizando los vuelos en curso.

use std::sync::Arc;

use eframe::egui::{Color32, Context, Painter, Response, Shape, Stroke};
use walkers::{extras::Image, Plugin, Position, Projector, Texture};

use crate::{
    data::airports::airp::{Airport, AirportsMap},
    interface::plugins::{flights::loader::LiveDataMap, utils::load_egui_img},
};

/// La ruta de la imagen de un vuelo en curso.
const IN_COURSE_IMG: &str = "media/img/airports/in_course.png";

/// Actualizador de aeropuertos.
pub struct FlightsUpdater {
    /// El contexto acutal.
    pub ctx: Context,

    /// Todos los aeropuertos disponibles.
    all_airports: AirportsMap,

    /// El puerto seleccionado actualmente.
    selected_airport: Arc<Option<Airport>>,

    /// Los datos de vuelos entrantes actualmente en memoria.
    incoming_tracking: Arc<LiveDataMap>,

    /// Los datos de vuelos salientes actualmente en memoria.
    departing_tracking: Arc<LiveDataMap>,

    /// La imagen de un vuelo en curso.
    in_course_img: Option<Texture>,
}

impl FlightsUpdater {
    /// Crea una nueva instancia del simulador de vuelos.
    pub fn new(
        ctx: Context,
        incoming_tracking: Arc<LiveDataMap>,
        departing_tracking: Arc<LiveDataMap>,
    ) -> Self {
        let text = Self::load_inc_course_img(&ctx);
        Self {
            ctx,
            all_airports: Airport::get_all().unwrap_or_default(),
            selected_airport: Arc::new(None),
            incoming_tracking,
            departing_tracking,
            in_course_img: text,
        }
    }

    /// Crea una nueva instancia con el contexto actual.
    pub fn with_ctx(ctx: Context) -> Self {
        Self::new(
            ctx,
            Arc::new(LiveDataMap::new()),
            Arc::new(LiveDataMap::new()),
        )
    }

    /// Carga la imagen de vuelo en curso.
    pub fn load_inc_course_img(ctx: &Context) -> Option<Texture> {
        match load_egui_img(IN_COURSE_IMG) {
            Err(_) => None,
            Ok(color_img) => Some(Texture::from_color_image(color_img, ctx)),
        }
    }

    /// Sincroniza el aeropuerto actual.
    pub fn sync_airport(&mut self, new_airport: Arc<Option<Airport>>) -> &mut Self {
        if new_airport.is_some() {
            self.selected_airport = new_airport;
        }

        self
    }

    /// Sincroniza los datos de vuelo entrantes.
    pub fn sync_incoming_tracking(&mut self, new_inc_tracking: Arc<LiveDataMap>) -> &mut Self {
        if !new_inc_tracking.is_empty() {
            self.incoming_tracking = new_inc_tracking;
        }

        self
    }

    /// Sincroniza los datos de vuelo salientes.
    pub fn sync_departing_tracking(&mut self, new_dep_tracking: Arc<LiveDataMap>) -> &mut Self {
        if !new_dep_tracking.is_empty() {
            self.departing_tracking = new_dep_tracking;
        }

        self
    }
}

impl Plugin for &mut FlightsUpdater {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        if let Some(airport) = self.selected_airport.as_ref() {
            for inc_data in self.incoming_tracking.values() {
                if let Some(last_entry) = inc_data.last() {
                    if last_entry.orig != airport.ident {
                        continue;
                    }
                    if let Some(dest) = self.all_airports.get(&last_entry.dest) {
                        let (orig_lat, orig_lon) = airport.position;
                        let (dest_lat, dest_lon) = dest.position;
                        let (cur_lat, cur_lon) = last_entry.pos;

                        let orig_pos = Position::from_lat_lon(orig_lat, orig_lon);
                        let dest_pos = Position::from_lat_lon(dest_lat, dest_lon);
                        let cur_pos = Position::from_lat_lon(cur_lat, cur_lon);

                        let mut orig_to_fl = Shape::dashed_line(
                            &[
                                projector.project(orig_pos).to_pos2(),
                                projector.project(cur_pos).to_pos2(),
                            ],
                            Stroke::new(20.0, Color32::from_rgb(50, 50, 50)),
                            10.0,
                            10.0,
                        );
                        let fl_to_dest = Shape::line_segment(
                            [
                                projector.project(cur_pos).to_pos2(),
                                projector.project(dest_pos).to_pos2(),
                            ],
                            Stroke::new(20.0, Color32::from_rgb(255, 60, 60)),
                        );

                        let mut drawable = Vec::<Shape>::new();
                        drawable.append(&mut orig_to_fl);
                        drawable.push(fl_to_dest);

                        painter.extend(drawable);

                        if let Some(text) = &self.in_course_img {
                            let mut img = Image::new(text.clone(), cur_pos);
                            img.scale(0.025, 0.025);
                            // TODO: rotarla según el ángulo entre las dos posiciones

                            img.draw(response, painter.clone(), projector);
                        }
                    }
                }
            }
        }
    }
}
