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

    /// Los vuelso salientes actualmente en memoria.
    departing_flights: Option<Vec<Flight>>,

    /// El tiempo que pasó desde la última _query_.
    last_checked: Instant,

    /// Fecha seleccionada.
    date: DateTime<Local>,

    /// Hilo hijo para cargar vuelos entrantes.
    incoming_child: (DateChild, Receiver<Vec<Flight>>),

    /// Hilo hijo para cargar vuelos salientes.
    departing_child: (DateChild, Receiver<Vec<Flight>>),
}

impl FlightsLoader {
    /// Crea una nueva instancia de cargador de vuelos.
    pub fn new(
        client: Arc<Mutex<Client>>,
        selected_airport: Arc<Option<Airport>>,
        flights: (Option<Vec<Flight>>, Option<Vec<Flight>>),
        last_checked: Instant,
        date: DateTime<Local>,
        children: (
            (DateChild, Receiver<Vec<Flight>>),
            (DateChild, Receiver<Vec<Flight>>),
        ),
    ) -> Self {
        let (incoming_flights, departing_flights) = flights;
        let (incoming_child, departing_child) = children;

        Self {
            client,
            selected_airport,
            incoming_flights,
            departing_flights,
            last_checked,
            date,
            incoming_child,
            departing_child,
        }
    }

    /// Genera el hilo hijo para cargar vuelos entrantes.
    pub fn gen_incoming_child(
        to_parent: Sender<Vec<Flight>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_child(to_parent, FlightType::Incoming, client)
    }

    /// Genera el hilo hijo para cargar vuelos salientes.
    pub fn gen_departing_child(
        to_parent: Sender<Vec<Flight>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_child(to_parent, FlightType::Departing, client)
    }

    /// Genera un hilo hijo.
    fn gen_date_child(
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

    /// **Consume** la lista de vuelos entrantes actualmente en memoria para devolverla,
    /// y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_incoming(&mut self) -> Vec<Flight> {
        self.incoming_flights.take().unwrap_or_default()
    }

    /// **Consume** la lista de vuelos salientes actualmente en memoria para devolverla,
    /// y en su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_departing(&mut self) -> Vec<Flight> {
        self.departing_flights.take().unwrap_or_default()
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
        let flights = Flight::try_from_protocol_result(protocol_result, &flight_type)?;

        Ok(flights)
    }

    /// Apaga y espera a todos los hilos hijos.
    pub fn wait_children(&mut self) {
        Self::wait_for_child(&mut self.incoming_child.0);
        Self::wait_for_child(&mut self.departing_child.0);
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
        let (incoming_sender, incoming_receiver) = channel::<Vec<Flight>>();
        let (departing_sender, departing_receiver) = channel::<Vec<Flight>>();

        let incoming_client = Arc::clone(&client);
        let departing_client = Arc::clone(&client);

        Self::new(
            client,
            Arc::new(None),
            (Some(Vec::<Flight>::new()), Some(Vec::<Flight>::new())),
            Instant::now(),
            Local::now(),
            (
                (
                    Self::gen_incoming_child(incoming_sender.clone(), incoming_client),
                    incoming_receiver,
                ),
                (
                    Self::gen_departing_child(departing_sender.clone(), departing_client),
                    departing_receiver,
                ),
            ),
        )
    }
}

impl Plugin for &mut FlightsLoader {
    fn run(&mut self, _response: &Response, _painter: Painter, _projector: &Projector) {
        if self.elapsed_at_least(&Duration::from_secs(FLIGHTS_INTERVAL_SECS)) {
            self.reset_instant();

            let ((_, incoming_sender), _) = &mut self.incoming_child;
            if let Err(err) =
                incoming_sender.send((Arc::clone(&self.selected_airport), self.date.timestamp()))
            {
                println!(
                    "Error al enviar timestamp al cargador de vuelos entrantes:\n\n{}",
                    err
                );
            }

            let ((_, departing_sender), _) = &mut self.departing_child;
            if let Err(err) =
                departing_sender.send((Arc::clone(&self.selected_airport), self.date.timestamp()))
            {
                println!(
                    "Error al enviar timestamp al cargador de vuelos salientes:\n\n{}",
                    err
                );
            }
        }

        if let Ok(new_incoming) = self.incoming_child.1.try_recv() {
            if !new_incoming.is_empty() {
                self.incoming_flights = Some(new_incoming);
            }
        }

        if let Ok(new_departing) = self.departing_child.1.try_recv() {
            if !new_departing.is_empty() {
                self.departing_flights = Some(new_departing);
            }
        }
    }
}

impl Drop for FlightsLoader {
    fn drop(&mut self) {
        self.wait_children();
    }
}
