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
    data::flights::{
        departing::DepartingFlight, flight_type::FlightType, incoming::IncomingFlight,
        traits::Flight,
    },
    protocol::{
        aliases::{results::Result, types::Long},
        errors::error::Error,
    },
};

/// Un hilo destinado a procesos paralelos.
type ChildHandle = JoinHandle<Result<()>>;

/// El tipo de hilo hijo para cargar datos según fecha.
type DateChild = (Option<ChildHandle>, Sender<Long>);

/// Intervalo (en segundos) antes de cargar los vuelos de nuevo, como mínimo.
const FLIGHTS_INTERVAL_SECS: u64 = 5;

/// Un día en segundos.
const DAY_IN_SECONDS: i64 = 86400;

/// Cargador de vuelos.
pub struct FlightsLoader {
    /// El cliente para pedir las queries.
    client: Arc<Mutex<Client>>,

    /// Los vuelos entrantes actualmente en memoria.
    incoming_flights: Option<Vec<IncomingFlight>>,

    /// Los vuelso salientes actualmente en memoria.
    departing_flights: Option<Vec<DepartingFlight>>,

    /// El tiempo que pasó desde la última _query_.
    last_checked: Instant,

    /// Fecha seleccionada.
    date: DateTime<Local>,

    /// Hilo hijo para cargar vuelos entrantes.
    incoming_child: DateChild,

    /// Extremo de canal que recibe actualizaciones de los vuelos entrantes.
    incoming_receiver: Receiver<Vec<FlightType>>,

    /// Hilo hijo para cargar vuelos salientes.
    departing_child: DateChild,

    /// Extremo de canal que recibe actualizaciones de los vuelos entrantes.
    departing_receiver: Receiver<Vec<FlightType>>,
}

impl FlightsLoader {
    /// Crea una nueva instancia de cargador de vuelos.
    pub fn new(
        client: Arc<Mutex<Client>>,
        flights: (Option<Vec<IncomingFlight>>, Option<Vec<DepartingFlight>>),
        last_checked: Instant,
        date: DateTime<Local>,
        children: (DateChild, DateChild),
        receivers: (Receiver<Vec<FlightType>>, Receiver<Vec<FlightType>>),
    ) -> Self {
        let (incoming_receiver, departing_receiver) = receivers;
        let (incoming_flights, departing_flights) = flights;
        let (incoming_child, departing_child) = children;

        Self {
            client,
            incoming_flights,
            departing_flights,
            last_checked,
            date,
            incoming_child,
            incoming_receiver,
            departing_child,
            departing_receiver,
        }
    }

    /// Genera el hilo hijo para cargar vuelos entrantes.
    pub fn gen_incoming_child(
        to_parent: Sender<Vec<FlightType>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_child(
            to_parent,
            FlightType::Incoming(IncomingFlight::dummy()),
            client,
        )
    }

    /// Genera el hilo hijo para cargar vuelos salientes.
    pub fn gen_departing_child(
        to_parent: Sender<Vec<FlightType>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_child(
            to_parent,
            FlightType::Departing(DepartingFlight::dummy()),
            client,
        )
    }

    /// Genera un hilo hijo.
    fn gen_date_child(
        to_parent: Sender<Vec<FlightType>>,
        flight_type: FlightType,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        let (date_sender, date_receiver) = channel::<Long>();
        let date_handle = spawn(move || {
            let stop_value: Long = 0;

            loop {
                match date_receiver.recv() {
                    Ok(timestamp) => {
                        if timestamp == stop_value {
                            break;
                        }
                        let flights =
                            match Self::load_flights(Arc::clone(&client), &flight_type, &timestamp)
                            {
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

    /// **Consume** la lista de vuelos entrantes actualmente en memoria para devolverla,
    /// y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_incoming(&mut self) -> Vec<IncomingFlight> {
        self.incoming_flights.take().unwrap_or_default()
    }

    /// **Consume** la lista de vuelos salientes actualmente en memoria para devolverla,
    /// y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_departing(&mut self) -> Vec<DepartingFlight> {
        self.departing_flights.take().unwrap_or_default()
    }

    /// Sincroniza la fecha seleccionada en la aplicación con la guardada aquí.
    pub fn sync_date(&mut self, new_date: DateTime<Local>) -> &mut Self {
        self.date = new_date;
        self
    }

    /// Sincroniza el cliente.
    pub fn sync_client(&mut self, new_client: Client) -> &mut Self {
        self.client = Arc::new(Mutex::new(new_client));
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
        timestamp: &Long,
    ) -> Result<Vec<FlightType>> {
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

        let query = match flight_type {
            FlightType::Incoming(_) => format!(
                "SELECT * FROM vuelos_entrantes WHERE llegada < {} AND llegada > {};",
                timestamp + (DAY_IN_SECONDS / 2),
                timestamp - (DAY_IN_SECONDS / 2),
            ),
            FlightType::Departing(_) => format!(
                "SELECT * FROM vuelos_salientes WHERE salida < {} AND salida > {};",
                timestamp + (DAY_IN_SECONDS / 2),
                timestamp - (DAY_IN_SECONDS / 2),
            ),
        };

        let protocol_result = client.send_query(query.as_str(), &mut tcp_stream)?;
        let flights = match flight_type {
            FlightType::Incoming(_) => IncomingFlight::try_from_protocol_result(protocol_result)?
                .into_iter()
                .map(FlightType::Incoming)
                .collect(),
            FlightType::Departing(_) => DepartingFlight::try_from_protocol_result(protocol_result)?
                .into_iter()
                .map(FlightType::Departing)
                .collect(),
        };

        println!("Vuelos de tipo '{:?}':\t{:?}", flight_type, flights);

        Ok(flights)
    }

    /// Apaga y espera a todos los hilos hijos.
    pub fn wait_children(&mut self) {
        Self::wait_for_child(&mut self.incoming_child);
        Self::wait_for_child(&mut self.departing_child);
    }

    /// Espera a un hijo específico.
    fn wait_for_child(child: &mut DateChild) {
        let (date_child, date_sender) = child;
        if let Some(hanging) = date_child.take() {
            if date_sender.send(0).is_err() {
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
        let (incoming_sender, incoming_receiver) = channel::<Vec<FlightType>>();
        let (departing_sender, departing_receiver) = channel::<Vec<FlightType>>();

        let incoming_client = Arc::clone(&client);
        let departing_client = Arc::clone(&client);

        Self::new(
            client,
            (
                Some(Vec::<IncomingFlight>::new()),
                Some(Vec::<DepartingFlight>::new()),
            ),
            Instant::now(),
            Local::now(),
            (
                Self::gen_incoming_child(incoming_sender.clone(), incoming_client),
                Self::gen_departing_child(departing_sender.clone(), departing_client),
            ),
            (incoming_receiver, departing_receiver),
        )
    }
}

impl Plugin for &mut FlightsLoader {
    fn run(&mut self, _response: &Response, _painter: Painter, _projector: &Projector) {
        if self.elapsed_at_least(&Duration::from_secs(FLIGHTS_INTERVAL_SECS)) {
            self.reset_instant();

            let (_, incoming_sender) = &mut self.incoming_child;
            if let Err(err) = incoming_sender.send(self.date.timestamp()) {
                println!(
                    "Error al enviar timestamp al cargador de vuelos entrantes:\n\n{}",
                    err
                );
            }

            let (_, departing_sender) = &mut self.departing_child;
            if let Err(err) = departing_sender.send(self.date.timestamp()) {
                println!(
                    "Error al enviar timestamp al cargador de vuelos salientes:\n\n{}",
                    err
                );
            }
        }

        if let Ok(new_incoming) = self.incoming_receiver.try_recv() {
            if !new_incoming.is_empty() {
                self.incoming_flights = Some(FlightType::filter_incoming(new_incoming));
            }
        }

        if let Ok(new_departing) = self.departing_receiver.try_recv() {
            if !new_departing.is_empty() {
                self.departing_flights = Some(FlightType::filter_departing(new_departing));
            }
        }
    }
}

impl Drop for FlightsLoader {
    fn drop(&mut self) {
        self.wait_children();
    }
}
