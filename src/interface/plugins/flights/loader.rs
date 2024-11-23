//! Módulo para un cargador de vuelos.

use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{spawn, JoinHandle},
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use eframe::egui::{Painter, Response};
use walkers::{Plugin, Projector};

use crate::{
    client::cli::Client,
    data::{
        airports::airp::Airport,
        flights::{flight::Flight, types::FlightType},
        tracking::live_flight_data::LiveFlightData,
    },
    protocol::{
        aliases::{results::Result, types::Long},
        errors::error::Error,
    },
};

/// Un hilo destinado a procesos paralelos.
type ChildHandle = JoinHandle<Result<()>>;

/// El tipo de hilo hijo para cargar datos según fecha.
type DateChild = (Option<ChildHandle>, Sender<(Arc<Option<Airport>>, Long)>);

/// Tipo de hilo hijo para vuelos.
type FlightChild = (DateChild, Receiver<Vec<Flight>>);

/// Tupi de hilo hijo para datos de vuelos.
type FlightDataChild = (DateChild, Receiver<Vec<LiveFlightData>>);

/// Intervalo (en segundos) antes de cargar los vuelos de nuevo, como mínimo.
const FLIGHTS_INTERVAL_SECS: u64 = 3;

/// Un día en segundos.
const DAY_IN_SECONDS: i64 = 86400;

/// Cargador de vuelos.
pub struct FlightsLoader {
    /// El cliente para pedir las queries.
    client: Arc<Mutex<Client>>,

    /// El aeropuerto acualmente seleccionado.
    selected_airport: Arc<Option<Airport>>,

    /// Los vuelos entrantes actualmente en memoria.
    incoming_flights: Option<Vec<Flight>>,

    /// Los vuelos salientes actualmente en memoria.
    departing_flights: Option<Vec<Flight>>,

    /// Los datos de vuelos entrantes actualmente en memoria.
    incoming_tracking: Option<Vec<LiveFlightData>>,

    /// Los datos de vuelos salientes actualmente en memoria.
    departing_tracking: Option<Vec<LiveFlightData>>,

    /// El tiempo que pasó desde la última _query_.
    last_checked: Instant,

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
        client: Arc<Mutex<Client>>,
        selected_airport: Arc<Option<Airport>>,
        flights: (Option<Vec<Flight>>, Option<Vec<Flight>>),
        tracking: (Option<Vec<LiveFlightData>>, Option<Vec<LiveFlightData>>),
        last_checked: Instant,
        date: DateTime<Local>,
        children: (FlightChild, FlightChild, FlightDataChild, FlightDataChild),
    ) -> Self {
        let (incoming_flights, departing_flights) = flights;
        let (incoming_tracking, departing_tracking) = tracking;
        let (incoming_fl_child, departing_fl_child, incoming_tr_child, departing_tr_child) =
            children;

        Self {
            client,
            selected_airport,
            incoming_flights,
            departing_flights,
            incoming_tracking,
            departing_tracking,
            last_checked,
            date,
            incoming_fl_child,
            departing_fl_child,
            incoming_tr_child,
            departing_tr_child,
        }
    }

    /// Genera el hilo hijo para cargar vuelos entrantes.
    pub fn gen_inc_fl_child(
        to_parent: Sender<Vec<Flight>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_fl_child(to_parent, FlightType::Incoming, client)
    }

    /// Genera el hilo hijo para cargar vuelos salientes.
    pub fn gen_dep_fl_child(
        to_parent: Sender<Vec<Flight>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_fl_child(to_parent, FlightType::Departing, client)
    }

    /// Genera un hilo hijo para vuelos.
    fn gen_date_fl_child(
        to_parent: Sender<Vec<Flight>>,
        flight_type: FlightType,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        let (date_sender, date_receiver) = channel::<(Arc<Option<Airport>>, Long)>();
        let date_handle = spawn(move || {
            let stop_value: Long = 0;
            let airport_stop = Airport::dummy();

            loop {
                match date_receiver.recv() {
                    Ok((selected_airport, timestamp)) => {
                        let mut stop_by_airport = false;
                        if let Some(airport) = selected_airport.as_ref() {
                            stop_by_airport = airport == &airport_stop;
                        }

                        if stop_by_airport && (timestamp == stop_value) {
                            break;
                        }
                        let flights = match Self::load_flights(
                            Arc::clone(&client),
                            &flight_type,
                            selected_airport.as_ref(),
                            &timestamp,
                        ) {
                            Ok(loaded) => loaded,
                            Err(err) => {
                                println!("Error cargando los vuelos:\n\n{}", err);
                                Vec::new()
                            }
                        };

                        if let Err(err) = to_parent.send(flights) {
                            println!("Error al mandar a hilo principal los vuelos:\n\n{}", err);
                        }
                    }
                    Err(err) => println!(
                        "Ocurrió un error esperando mensajes del hilo principal:\n\n{}",
                        err
                    ),
                }
            }

            Ok(())
        });

        (Some(date_handle), date_sender.clone())
    }

    /// Genera el hilo hijo para cargar datos de vuelos entrantes.
    pub fn gen_inc_tr_child(
        to_parent: Sender<Vec<LiveFlightData>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_tr_child(to_parent, FlightType::Incoming, client)
    }

    /// Genera el hilo hijo para cargar datos de vuelos salientes.
    pub fn gen_dep_tr_child(
        to_parent: Sender<Vec<LiveFlightData>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_tr_child(to_parent, FlightType::Departing, client)
    }

    /// Genera un hilo hijo para datos de vuelos.
    fn gen_date_tr_child(
        to_parent: Sender<Vec<LiveFlightData>>,
        flight_type: FlightType,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        let (date_sender, date_receiver) = channel::<(Arc<Option<Airport>>, Long)>();
        let date_handle = spawn(move || {
            let stop_value: Long = 0;
            let airport_stop = Airport::dummy();

            loop {
                match date_receiver.recv() {
                    Ok((selected_airport, timestamp)) => {
                        let mut stop_by_airport = false;
                        if let Some(airport) = selected_airport.as_ref() {
                            stop_by_airport = airport == &airport_stop;
                        }

                        if stop_by_airport && (timestamp == stop_value) {
                            break;
                        }
                        let live_data = match Self::load_live_data(
                            Arc::clone(&client),
                            &flight_type,
                            selected_airport.as_ref(),
                        ) {
                            Ok(loaded) => loaded,
                            Err(err) => {
                                println!("Error cargando los vuelos:\n\n{}", err);
                                Vec::new()
                            }
                        };

                        if let Err(err) = to_parent.send(live_data) {
                            println!("Error al mandar a hilo principal los vuelos:\n\n{}", err);
                        }
                    }
                    Err(err) => println!(
                        "Ocurrió un error esperando mensajes del hilo principal:\n\n{}",
                        err
                    ),
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
        self.incoming_flights.take().unwrap_or_default()
    }

    /// **Consume** la lista de vuelos salientes actualmente en memoria para devolverla,
    /// y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_fl_departing(&mut self) -> Vec<Flight> {
        self.departing_flights.take().unwrap_or_default()
    }

    /// **Consume** la lista de datos de vuelos entrantes actualmente en memoria
    /// para devolverla, y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_tr_incoming(&mut self) -> Vec<LiveFlightData> {
        self.incoming_tracking.take().unwrap_or_default()
    }

    /// **Consume** la lista de datos de vuelos salientes actualmente en memoria
    /// para devolverla, y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_tr_departing(&mut self) -> Vec<LiveFlightData> {
        self.departing_tracking.take().unwrap_or_default()
    }

    /// Sincroniza la fecha seleccionada en la aplicación con la guardada aquí.
    pub fn sync_date(&mut self, new_date: DateTime<Local>) -> &mut Self {
        self.date = new_date;
        self
    }

    /// Sincroniza el cliente.
    pub fn sync_client(&mut self, new_client: Arc<Mutex<Client>>) -> &mut Self {
        self.client = new_client;
        self
    }

    /// Sincroniza el aeropuerto seleccionado.
    pub fn sync_selected_airport(&mut self, new_airport: Arc<Option<Airport>>) -> &mut Self {
        self.selected_airport = new_airport;
        self
    }

    /// Resetea el chequeo al [Instant] actual.
    pub fn reset_instant(&mut self) {
        self.last_checked = Instant::now();
    }

    /// Verifica si ha pasado un mínimo de tiempo dado desde la última vez
    /// que se editaron los vuelos.
    pub fn elapsed_at_least(&self, duration: &Duration) -> bool {
        &self.last_checked.elapsed() >= duration
    }

    /// Carga los vuelos con una _query_.
    fn load_flights(
        client_lock: Arc<Mutex<Client>>,
        flight_type: &FlightType,
        selected_airport: &Option<Airport>,
        timestamp: &Long,
    ) -> Result<Vec<Flight>> {
        let mut client = match client_lock.lock() {
            Err(poison_err) => {
                return Err(Error::ServerError(format!(
                    "Error de lock envenenado al cargar vuelos:\n\n{}",
                    poison_err
                )));
            }
            Ok(cli) => cli,
        };

        let mut tcp_stream = client.connect()?;

        let airport = match selected_airport {
            Some(airp) => airp,
            None => return Ok(Vec::<Flight>::new()),
        };
        let query = match flight_type {
            FlightType::Incoming => format!(
                "SELECT * FROM vuelos_entrantes WHERE dest = '{}' AND llegada < {} AND llegada > {};",
                airport.ident,
                timestamp + (DAY_IN_SECONDS / 2),
                timestamp - (DAY_IN_SECONDS / 2),
            ),
            FlightType::Departing => format!(
                "SELECT * FROM vuelos_salientes WHERE orig = '{}' AND salida < {} AND salida > {};",
                airport.ident,
                timestamp + (DAY_IN_SECONDS / 2),
                timestamp - (DAY_IN_SECONDS / 2),
            ),
        };

        let protocol_result = client.send_query(query.as_str(), &mut tcp_stream)?;
        let flights = Flight::try_from_protocol_result(protocol_result, flight_type)?;

        Ok(flights)
    }

    /// Carga los datos de vuelos con una _query_.
    fn load_live_data(
        client_lock: Arc<Mutex<Client>>,
        flight_type: &FlightType,
        selected_airport: &Option<Airport>,
    ) -> Result<Vec<LiveFlightData>> {
        let mut client = match client_lock.lock() {
            Err(poison_err) => {
                return Err(Error::ServerError(format!(
                    "Error de lock envenenado al cargar vuelos:\n\n{}",
                    poison_err
                )));
            }
            Ok(cli) => cli,
        };

        let mut tcp_stream = client.connect()?;

        let airport = match selected_airport {
            Some(airp) => airp,
            None => return Ok(Vec::<LiveFlightData>::new()),
        };
        let query = match flight_type {
            FlightType::Incoming => format!(
                "SELECT * FROM vuelos_entrantes_en_vivo WHERE dest = '{}';",
                airport.ident
            ),
            FlightType::Departing => format!(
                "SELECT * FROM vuelos_salientes_en_vivo WHERE orig = '{}';",
                airport.ident
            ),
        };

        let protocol_result = client.send_query(query.as_str(), &mut tcp_stream)?;
        let live_data = LiveFlightData::try_from_protocol_result(protocol_result, flight_type)?;

        Ok(live_data)
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
                .send((Arc::new(Some(Airport::dummy())), 0))
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
        let client = Arc::new(Mutex::new(Client::default()));
        let (inc_fl_sender, inc_fl_receiver) = channel::<Vec<Flight>>();
        let (dep_fl_sender, dep_fl_receiver) = channel::<Vec<Flight>>();
        let (inc_tr_sender, inc_tr_receiver) = channel::<Vec<LiveFlightData>>();
        let (dep_tr_sender, dep_tr_receiver) = channel::<Vec<LiveFlightData>>();

        let inc_fl_cli = Arc::clone(&client);
        let dep_fl_cli = Arc::clone(&client);
        let inc_tr_cli = Arc::clone(&client);
        let dep_tr_cli = Arc::clone(&client);

        Self::new(
            client,
            Arc::new(None),
            (Some(Vec::<Flight>::new()), Some(Vec::<Flight>::new())),
            (
                Some(Vec::<LiveFlightData>::new()),
                Some(Vec::<LiveFlightData>::new()),
            ),
            Instant::now(),
            Local::now(),
            (
                (
                    Self::gen_inc_fl_child(inc_fl_sender.clone(), inc_fl_cli),
                    inc_fl_receiver,
                ),
                (
                    Self::gen_dep_fl_child(dep_fl_sender.clone(), dep_fl_cli),
                    dep_fl_receiver,
                ),
                (
                    Self::gen_inc_tr_child(inc_tr_sender.clone(), inc_tr_cli),
                    inc_tr_receiver,
                ),
                (
                    Self::gen_dep_tr_child(dep_tr_sender.clone(), dep_tr_cli),
                    dep_tr_receiver,
                ),
            ),
        )
    }
}

impl Plugin for &mut FlightsLoader {
    fn run(&mut self, _response: &Response, _painter: Painter, _projector: &Projector) {
        if self.elapsed_at_least(&Duration::from_secs(FLIGHTS_INTERVAL_SECS)) {
            self.reset_instant();

            let ((_, inc_fl_sender), _) = &mut self.incoming_fl_child;
            if let Err(err) =
                inc_fl_sender.send((Arc::clone(&self.selected_airport), self.date.timestamp()))
            {
                println!(
                    "Error al enviar timestamp al cargador de vuelos entrantes:\n\n{}",
                    err
                );
            }

            let ((_, dep_fl_sender), _) = &mut self.departing_fl_child;
            if let Err(err) =
                dep_fl_sender.send((Arc::clone(&self.selected_airport), self.date.timestamp()))
            {
                println!(
                    "Error al enviar timestamp al cargador de vuelos salientes:\n\n{}",
                    err
                );
            }

            let ((_, inc_tr_sender), _) = &mut self.incoming_tr_child;
            if let Err(err) =
                inc_tr_sender.send((Arc::clone(&self.selected_airport), self.date.timestamp()))
            {
                println!(
                    "Error al enviar timestamp al cargador de datos de vuelos entrantes:\n\n{}",
                    err
                );
            }

            let ((_, dep_tr_sender), _) = &mut self.departing_tr_child;
            if let Err(err) =
                dep_tr_sender.send((Arc::clone(&self.selected_airport), self.date.timestamp()))
            {
                println!(
                    "Error al enviar timestamp al cargador de datos de vuelos salientes:\n\n{}",
                    err
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
