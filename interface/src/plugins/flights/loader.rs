//! Módulo para un cargador de vuelos.

use {
    chrono::{DateTime, Local},
    client::{cli::Client, conn_holder::ConnectionHolder},
    data::{
        airports::airp::Airport,
        flights::{flight::Flight, types::FlightType},
        login_info::LoginInfo,
        tracking::live_flight_data::LiveFlightData,
    },
    eframe::egui::{Painter, Response},
    protocol::{
        aliases::{
            results::Result,
            types::{Int, Long, Ulong},
        },
        errors::error::Error,
    },
    std::{
        collections::hash_map::{Entry, HashMap},
        sync::{
            mpsc::{channel, Receiver, Sender},
            Arc,
        },
        thread::{spawn, JoinHandle},
        time::{Duration, Instant},
    },
    walkers::{Plugin, Projector},
};

/// Los datos de vuelos ordenados por ID.
pub type LiveDataMap = HashMap<Int, Vec<LiveFlightData>>;

/// Un hilo destinado a procesos paralelos.
type ChildHandle = JoinHandle<Result<()>>;

/// El tipo de hilo hijo para cargar datos según fecha.
type DateChild = (
    Option<ChildHandle>,
    Sender<(Arc<Option<Airport>>, Long, Option<LoginInfo>)>,
);

/// Tipo de hilo hijo para vuelos.
type FlightChild = (DateChild, Receiver<Vec<Flight>>);

/// Tupi de hilo hijo para datos de vuelos.
type FlightDataChild = (DateChild, Receiver<LiveDataMap>);

/// Intervalo (en segundos) antes de cargar los vuelos de nuevo, como mínimo.
const FLIGHTS_INTERVAL_SECS: Ulong = 3;
/// Intervalo (en segundos) antes de cargar los datos de vuelos de nuevo, como mínimo.
const TRACKING_INTERVAL_SECS: Ulong = 1;

/// Un día en segundos.
const DAY_IN_SECONDS: Long = 86400;

/// Cargador de vuelos.
pub struct FlightsLoader {
    /// La información de logueo para conectarse.
    login_info: LoginInfo,

    /// Si se debe reloguear en la conexión para los vuelos.
    relogin_fl: bool,

    /// Si se debe reloguear en la conexión para los datos de vuelos.
    relogin_tr: bool,

    /// El aeropuerto acualmente seleccionado.
    selected_airport: Arc<Option<Airport>>,

    /// Los vuelos entrantes actualmente en memoria.
    incoming_flights: Option<Vec<Flight>>,

    /// Los vuelos salientes actualmente en memoria.
    departing_flights: Option<Vec<Flight>>,

    /// Los datos de vuelos entrantes actualmente en memoria.
    incoming_tracking: Option<LiveDataMap>,

    /// Los datos de vuelos salientes actualmente en memoria.
    departing_tracking: Option<LiveDataMap>,

    /// El tiempo que pasó desde la última _query_ para vuelos.
    last_checked_fl: Instant,

    /// El tiempo que pasó desde la última _query_ para datos de vuelos.
    last_checked_tr: Instant,

    /// Fecha seleccionada.
    date: DateTime<Local>,

    /// Hilo hijo para cargar vuelos entrantes.
    incoming_fl_child: FlightChild,

    /// Hilo hijo para cargar vuelos salientes.
    departing_fl_child: FlightChild,

    /// Hilo hijo para cargar datos de vuelos entrantes.
    incoming_tr_child: FlightDataChild,

    /// Hilo hijo para cargar datos de vuelos salientes.
    departing_tr_child: FlightDataChild,
}

impl FlightsLoader {
    /// Crea una nueva instancia de cargador de vuelos.
    pub fn new(
        login_info: LoginInfo,
        selected_airport: Arc<Option<Airport>>,
        flights: (Option<Vec<Flight>>, Option<Vec<Flight>>),
        tracking: (Option<LiveDataMap>, Option<LiveDataMap>),
        last_checked: (Instant, Instant),
        date: DateTime<Local>,
        children: (FlightChild, FlightChild, FlightDataChild, FlightDataChild),
    ) -> Self {
        let (incoming_flights, departing_flights) = flights;
        let (incoming_tracking, departing_tracking) = tracking;
        let (last_checked_fl, last_checked_tr) = last_checked;
        let (incoming_fl_child, departing_fl_child, incoming_tr_child, departing_tr_child) =
            children;

        Self {
            login_info,
            relogin_fl: false,
            relogin_tr: false,
            selected_airport,
            incoming_flights,
            departing_flights,
            incoming_tracking,
            departing_tracking,
            last_checked_fl,
            last_checked_tr,
            date,
            incoming_fl_child,
            departing_fl_child,
            incoming_tr_child,
            departing_tr_child,
        }
    }

    /// Genera el hilo hijo para cargar vuelos entrantes.
    pub fn gen_inc_fl_child(to_parent: Sender<Vec<Flight>>) -> DateChild {
        Self::gen_date_fl_child(to_parent, FlightType::Incoming)
    }

    /// Genera el hilo hijo para cargar vuelos salientes.
    pub fn gen_dep_fl_child(to_parent: Sender<Vec<Flight>>) -> DateChild {
        Self::gen_date_fl_child(to_parent, FlightType::Departing)
    }

    /// Genera un hilo hijo para vuelos.
    fn gen_date_fl_child(to_parent: Sender<Vec<Flight>>, flight_type: FlightType) -> DateChild {
        let (date_sender, date_receiver) =
            channel::<(Arc<Option<Airport>>, Long, Option<LoginInfo>)>();
        let date_handle = spawn(move || {
            let stop_value: Long = 0;
            let airport_stop = Airport::dummy();
            let mut con_info = ConnectionHolder::with_cli(Client::default(), "QUORUM")?;

            loop {
                match date_receiver.recv() {
                    Ok((selected_airport, timestamp, login_info_opt)) => {
                        let mut stop_by_airport = false;
                        if let Some(airport) = selected_airport.as_ref() {
                            stop_by_airport = airport == &airport_stop;
                        }
                        if stop_by_airport && (timestamp == stop_value) {
                            break;
                        }

                        if let Some(login_info) = login_info_opt {
                            if let Err(login_err) = con_info.login(&login_info) {
                                println!("Error al loguearse en el hilo cargador:\n\n{login_err}");
                            }
                        }

                        let flights = match Self::load_flights(
                            &mut con_info,
                            &flight_type,
                            selected_airport.as_ref(),
                            &timestamp,
                        ) {
                            Ok(data) => data,
                            Err(err) => {
                                println!("Error cargando vuelos:\n{err}");
                                Vec::<Flight>::new()
                            }
                        };

                        if let Err(err) = to_parent.send(flights) {
                            println!("Error al mandar a hilo principal los vuelos:\n\n{err}");
                        }
                    }
                    Err(err) => {
                        println!("Ocurrió un error esperando mensajes del hilo principal:\n\n{err}")
                    }
                }
            }

            Ok(())
        });

        (Some(date_handle), date_sender.clone())
    }

    /// Genera el hilo hijo para cargar datos de vuelos entrantes.
    pub fn gen_inc_tr_child(to_parent: Sender<LiveDataMap>) -> DateChild {
        Self::gen_date_tr_child(to_parent, FlightType::Incoming)
    }

    /// Genera el hilo hijo para cargar datos de vuelos salientes.
    pub fn gen_dep_tr_child(to_parent: Sender<LiveDataMap>) -> DateChild {
        Self::gen_date_tr_child(to_parent, FlightType::Departing)
    }

    /// Genera un hilo hijo para datos de vuelos.
    fn gen_date_tr_child(to_parent: Sender<LiveDataMap>, flight_type: FlightType) -> DateChild {
        let (date_sender, date_receiver) =
            channel::<(Arc<Option<Airport>>, Long, Option<LoginInfo>)>();
        let date_handle = spawn(move || {
            let stop_value: Long = 0;
            let airport_stop = Airport::dummy();
            let mut con_info = ConnectionHolder::with_cli(Client::default(), "ONE")?;

            loop {
                match date_receiver.recv() {
                    Ok((selected_airport, timestamp, login_info_opt)) => {
                        let mut stop_by_airport = false;
                        if let Some(airport) = selected_airport.as_ref() {
                            stop_by_airport = airport == &airport_stop;
                        }
                        if stop_by_airport && (timestamp == stop_value) {
                            break;
                        }

                        if let Some(login_info) = login_info_opt {
                            if let Err(login_err) = con_info.login(&login_info) {
                                println!("Error al loguearse en el hilo cargador:\n\n{login_err}");
                            }
                        }

                        let live_data = match Self::load_live_data(
                            &mut con_info,
                            &flight_type,
                            selected_airport.as_ref(),
                        ) {
                            Ok(data) => data,
                            Err(err) => {
                                println!("Error cargando datos de vuelos en vivo:\n{err}");
                                LiveDataMap::new()
                            }
                        };

                        if let Err(err) = to_parent.send(live_data) {
                            println!("Error al mandar a hilo principal los vuelos:\n\n{err}");
                        }
                    }
                    Err(err) => {
                        println!("Ocurrió un error esperando mensajes del hilo principal:\n\n{err}")
                    }
                }
            }

            Ok(())
        });

        (Some(date_handle), date_sender.clone())
    }

    /// **Consume** la lista de vuelos entrantes actualmente en memoria para devolverla,
    /// y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_fl_incoming(&mut self) -> Vec<Flight> {
        self.incoming_flights
            .take()
            .map_or(Vec::<Flight>::new(), |flights| flights)
    }

    /// **Consume** la lista de vuelos salientes actualmente en memoria para devolverla,
    /// y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_fl_departing(&mut self) -> Vec<Flight> {
        self.departing_flights
            .take()
            .map_or(Vec::<Flight>::new(), |flights| flights)
    }

    /// **Consume** la lista de datos de vuelos entrantes actualmente en memoria
    /// para devolverla, y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_tr_incoming(&mut self) -> LiveDataMap {
        self.incoming_tracking
            .take()
            .map_or(LiveDataMap::new(), |flights| flights)
    }

    /// **Consume** la lista de datos de vuelos salientes actualmente en memoria
    /// para devolverla, y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_tr_departing(&mut self) -> LiveDataMap {
        self.departing_tracking
            .take()
            .map_or(LiveDataMap::new(), |flights| flights)
    }

    /// Sincroniza la fecha seleccionada en la aplicación con la guardada aquí.
    pub fn sync_date(&mut self, new_date: DateTime<Local>) -> &mut Self {
        self.date = new_date;
        self
    }

    /// Sincroniza la información de logueo.
    pub fn sync_login_info(&mut self, new_info: &LoginInfo) -> &mut Self {
        if new_info != &self.login_info {
            self.login_info = new_info.clone();
            self.relogin_fl = true;
            self.relogin_tr = true;
        }
        self
    }

    /// Sincroniza el aeropuerto seleccionado.
    pub fn sync_selected_airport(&mut self, new_airport: Arc<Option<Airport>>) -> &mut Self {
        self.selected_airport = new_airport;
        self
    }

    /// Resetea el chequeo al [Instant] actual de vuelos.
    pub fn reset_instant_fl(&mut self) {
        self.last_checked_fl = Instant::now();
    }

    /// Verifica si ha pasado un mínimo de tiempo dado desde la última vez
    /// que se editaron los vuelos.
    pub fn elapsed_at_least_fl(&self, duration: &Duration) -> bool {
        &self.last_checked_fl.elapsed() >= duration
    }

    /// Resetea el chequeo al [Instant] actual de datos de vuelos.
    pub fn reset_instant_tr(&mut self) {
        self.last_checked_tr = Instant::now();
    }

    /// Verifica si ha pasado un mínimo de tiempo dado desde la última vez
    /// que se editaron los datos de vuelos.
    pub fn elapsed_at_least_tr(&self, duration: &Duration) -> bool {
        &self.last_checked_tr.elapsed() >= duration
    }

    /// Carga los vuelos con una _query_.
    ///
    /// Se asume que en la conexión, uno ya se encuentra logueado.
    fn load_flights(
        con_info: &mut ConnectionHolder,
        flight_type: &FlightType,
        selected_airport: &Option<Airport>,
        timestamp: &Long,
    ) -> Result<Vec<Flight>> {
        let client_lock = con_info.get_cli();

        let mut client = match client_lock.lock() {
            Err(poison_err) => {
                client_lock.clear_poison();
                return Err(Error::ServerError(format!(
                    "Error de lock envenenado al cargar vuelos:\n\n{poison_err}"
                )));
            }
            Ok(cli) => cli,
        };

        let airport = match selected_airport {
            Some(airp) => airp,
            None => return Ok(Vec::<Flight>::new()),
        };

        let iata_code = match &airport.iata_code {
            Some(code) => code.to_string(),
            None => return Ok(Vec::<Flight>::new()),
        };

        let query = match flight_type {
            FlightType::Incoming => format!(
                "SELECT * FROM vuelos_entrantes WHERE dest = '{}' AND llegada < {} AND llegada > {};",
                iata_code,
                timestamp + (DAY_IN_SECONDS / 2),
                timestamp - (DAY_IN_SECONDS / 2),
            ),
            FlightType::Departing => format!(
                "SELECT * FROM vuelos_salientes WHERE orig = '{}' AND salida < {} AND salida > {};",
                iata_code,
                timestamp + (DAY_IN_SECONDS / 2),
                timestamp - (DAY_IN_SECONDS / 2),
            ),
        };

        let (protocol_result, mut new_tls_opt) =
            client.send_query(query.as_str(), &mut con_info.tls_stream)?;
        if let Some(new_tls) = new_tls_opt.take() {
            con_info.tls_stream = new_tls;
        }
        let flights = Flight::try_from_protocol_result(protocol_result, flight_type)?;

        Ok(flights)
    }

    /// Carga los datos de vuelos con una _query_.
    ///
    /// Se asume que en la conexión, uno ya se encuentra logueado.
    fn load_live_data(
        con_info: &mut ConnectionHolder,
        flight_type: &FlightType,
        selected_airport: &Option<Airport>,
    ) -> Result<LiveDataMap> {
        let client_lock = con_info.get_cli();

        let mut client = match client_lock.lock() {
            Err(poison_err) => {
                client_lock.clear_poison();
                return Err(Error::ServerError(format!(
                    "Error de lock envenenado al cargar vuelos:\n\n{poison_err}"
                )));
            }
            Ok(cli) => cli,
        };

        let airport = match selected_airport {
            Some(airp) => airp,
            None => return Ok(LiveDataMap::new()),
        };

        let iata_code = match &airport.iata_code {
            Some(code) => code.to_string(),
            None => return Ok(LiveDataMap::new()),
        };

        let query = match flight_type {
            FlightType::Incoming => {
                format!("SELECT * FROM vuelos_entrantes_en_vivo WHERE dest = '{iata_code}';")
            }
            FlightType::Departing => {
                format!("SELECT * FROM vuelos_salientes_en_vivo WHERE orig = '{iata_code}';")
            }
        };

        let mut flights_by_id = LiveDataMap::new();
        let (protocol_result, mut new_tls_opt) =
            client.send_query(query.as_str(), &mut con_info.tls_stream)?;
        if let Some(new_tls) = new_tls_opt.take() {
            con_info.tls_stream = new_tls;
        }

        let live_data = LiveFlightData::try_from_protocol_result(protocol_result, flight_type)?;
        for data in live_data {
            if let Entry::Vacant(entry) = flights_by_id.entry(data.flight_id) {
                entry.insert(Vec::<LiveFlightData>::new());
            }
            if let Some(entries) = flights_by_id.get_mut(&data.flight_id) {
                entries.push(data);
            }
        }

        Ok(flights_by_id)
    }

    /// Apaga y espera a todos los hilos hijos.
    pub fn wait_children(&mut self) {
        Self::wait_for_child(&mut self.incoming_fl_child.0);
        Self::wait_for_child(&mut self.departing_fl_child.0);
        Self::wait_for_child(&mut self.incoming_tr_child.0);
        Self::wait_for_child(&mut self.departing_tr_child.0);
    }

    /// Espera a un hijo específico.
    fn wait_for_child(child: &mut DateChild) {
        let (date_child, date_sender) = child;
        if let Some(hanging) = date_child.take() {
            if date_sender
                .send((Arc::new(Some(Airport::dummy())), 0, None))
                .is_err()
            {
                println!("Error mandando un mensaje para parar hilo de fecha.")
            }
            if hanging.join().is_err() {
                println!("Error esperando a que un hilo hijo termine.")
            }
        }
    }
}

impl Default for FlightsLoader {
    fn default() -> Self {
        let (inc_fl_sender, inc_fl_receiver) = channel::<Vec<Flight>>();
        let (dep_fl_sender, dep_fl_receiver) = channel::<Vec<Flight>>();
        let (inc_tr_sender, inc_tr_receiver) = channel::<LiveDataMap>();
        let (dep_tr_sender, dep_tr_receiver) = channel::<LiveDataMap>();

        Self::new(
            LoginInfo::default(),
            Arc::new(None),
            (Some(Vec::<Flight>::new()), Some(Vec::<Flight>::new())),
            (Some(LiveDataMap::new()), Some(LiveDataMap::new())),
            (Instant::now(), Instant::now()),
            Local::now(),
            (
                (
                    Self::gen_inc_fl_child(inc_fl_sender.clone()),
                    inc_fl_receiver,
                ),
                (
                    Self::gen_dep_fl_child(dep_fl_sender.clone()),
                    dep_fl_receiver,
                ),
                (
                    Self::gen_inc_tr_child(inc_tr_sender.clone()),
                    inc_tr_receiver,
                ),
                (
                    Self::gen_dep_tr_child(dep_tr_sender.clone()),
                    dep_tr_receiver,
                ),
            ),
        )
    }
}

impl Plugin for &mut FlightsLoader {
    fn run(&mut self, _response: &Response, _painter: Painter, _projector: &Projector) {
        let clone_dummy = self.login_info.to_owned(); // para no prestar mucho self
        let cloned_login_info = |relog| {
            if relog {
                Some(clone_dummy.clone())
            } else {
                None
            }
        };

        if self.elapsed_at_least_fl(&Duration::from_secs(FLIGHTS_INTERVAL_SECS)) {
            self.reset_instant_fl();
            let relogin_fl = match &self.relogin_fl {
                true => {
                    self.relogin_fl = false;
                    true
                }
                false => false,
            };

            let ((_, inc_fl_sender), _) = &mut self.incoming_fl_child;
            if let Err(err) = inc_fl_sender.send((
                Arc::clone(&self.selected_airport),
                self.date.timestamp(),
                cloned_login_info(relogin_fl),
            )) {
                println!("Error al enviar timestamp al cargador de vuelos entrantes:\n\n{err}");
            }

            let ((_, dep_fl_sender), _) = &mut self.departing_fl_child;
            if let Err(err) = dep_fl_sender.send((
                Arc::clone(&self.selected_airport),
                self.date.timestamp(),
                cloned_login_info(relogin_fl),
            )) {
                println!("Error al enviar timestamp al cargador de vuelos salientes:\n\n{err}");
            }
        }

        if self.elapsed_at_least_tr(&Duration::from_secs(TRACKING_INTERVAL_SECS)) {
            self.reset_instant_tr();
            let relogin_tr = match &self.relogin_tr {
                true => {
                    self.relogin_tr = false;
                    true
                }
                false => false,
            };

            let ((_, inc_tr_sender), _) = &mut self.incoming_tr_child;
            if let Err(err) = inc_tr_sender.send((
                Arc::clone(&self.selected_airport),
                self.date.timestamp(),
                cloned_login_info(relogin_tr),
            )) {
                println!(
                    "Error al enviar timestamp al cargador de datos de vuelos entrantes:\n\n{err}"
                );
            }

            let ((_, dep_tr_sender), _) = &mut self.departing_tr_child;
            if let Err(err) = dep_tr_sender.send((
                Arc::clone(&self.selected_airport),
                self.date.timestamp(),
                cloned_login_info(relogin_tr),
            )) {
                println!(
                    "Error al enviar timestamp al cargador de datos de vuelos salientes:\n\n{err}"
                );
            }
        }

        if let Ok(new_fl_incoming) = self.incoming_fl_child.1.try_recv() {
            if !new_fl_incoming.is_empty() {
                self.incoming_flights = Some(new_fl_incoming);
            }
        }

        if let Ok(new_fl_departing) = self.departing_fl_child.1.try_recv() {
            if !new_fl_departing.is_empty() {
                self.departing_flights = Some(new_fl_departing);
            }
        }

        if let Ok(new_tr_incoming) = self.incoming_tr_child.1.try_recv() {
            if !new_tr_incoming.is_empty() {
                self.incoming_tracking = Some(new_tr_incoming);
            }
        }

        if let Ok(new_tr_departing) = self.departing_tr_child.1.try_recv() {
            if !new_tr_departing.is_empty() {
                self.departing_tracking = Some(new_tr_departing);
            }
        }
    }
}

impl Drop for FlightsLoader {
    fn drop(&mut self) {
        self.wait_children();
    }
}
