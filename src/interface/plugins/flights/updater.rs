//! Módulo para plugin que va actualizando los vuelos en curso.

use {
    crate::{
        data::{
            airports::airp::{Airport, AirportsMap},
            tracking::live_flight_data::LiveFlightData,
        },
        interface::plugins::{flights::loader::LiveDataMap, utils::load_egui_img},
        protocol::aliases::{results::Result, types::Double},
    },
    eframe::egui::{Color32, Context, Painter, Response, Shape, Stroke},
    std::sync::Arc,
    walkers::{extras::Image, Plugin, Position, Projector, Texture},
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
    ) -> Result<Self> {
        let text = Self::load_inc_course_img(&ctx);
        Ok(Self {
            ctx,
            all_airports: Airport::get_all()?,
            selected_airport: Arc::new(None),
            incoming_tracking,
            departing_tracking,
            in_course_img: text,
        })
    }

    /// Crea una nueva instancia con el contexto actual.
    pub fn with_ctx(ctx: Context) -> Result<Self> {
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

    /// Dibuja la lína de viaje.
    pub fn draw_flight_line(
        &self,
        response: &Response,
        painter: Painter,
        projector: &Projector,
        orig_dest: ((Double, Double), (Double, Double)),
        cur_pos: (Double, Double),
        chosen_color: Color32,
    ) {
        let ((orig_lat, orig_lon), (dest_lat, dest_lon)) = orig_dest;
        let (cur_lat, cur_lon) = cur_pos;

        let orig_pos = Position::from_lat_lon(orig_lat, orig_lon);
        let dest_pos = Position::from_lat_lon(dest_lat, dest_lon);
        let cur_pos = Position::from_lat_lon(cur_lat, cur_lon);

        let mut orig_to_fl = Shape::dashed_line(
            &[
                projector.project(orig_pos).to_pos2(),
                projector.project(cur_pos).to_pos2(),
            ],
            Stroke::new(2.0, Color32::from_rgb(50, 50, 50)),
            10.0,
            10.0,
        );
        let fl_to_dest = Shape::line_segment(
            [
                projector.project(cur_pos).to_pos2(),
                projector.project(dest_pos).to_pos2(),
            ],
            Stroke::new(2.0, chosen_color),
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

impl Plugin for &mut FlightsUpdater {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        if let Some(airport) = self.selected_airport.as_ref() {
            for dep_data in self.departing_tracking.values() {
                if let Some(recent_entry) = LiveFlightData::most_recent(dep_data) {
                    let iata_code = match &airport.iata_code {
                        Some(code) => code.to_string(),
                        None => continue,
                    };

                    if recent_entry.orig != iata_code {
                        continue;
                    }

                    if let Some(dest) = self.all_airports.get(&recent_entry.dest) {
                        self.draw_flight_line(
                            response,
                            painter.clone(),
                            projector,
                            (airport.position, dest.position),
                            recent_entry.pos,
                            Color32::from_rgb(255, 60, 60),
                        );
                    }
                }
            }

            for inc_data in self.incoming_tracking.values() {
                if let Some(recent_entry) = LiveFlightData::most_recent(inc_data) {
                    let iata_code = match &airport.iata_code {
                        Some(code) => code.to_string(),
                        None => continue,
                    };

                    if recent_entry.dest != iata_code {
                        continue;
                    }

                    if let Some(orig) = self.all_airports.get(&recent_entry.orig) {
                        self.draw_flight_line(
                            response,
                            painter.clone(),
                            projector,
                            (orig.position, airport.position),
                            recent_entry.pos,
                            Color32::from_rgb(60, 60, 200),
                        );
                    }
                }
            }
        }
    }
}
