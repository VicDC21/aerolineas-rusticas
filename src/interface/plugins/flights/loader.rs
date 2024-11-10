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

use crate::interface::plugins::flights::flight_type::LoadableFlight;
use crate::{
    client::{cli::Client, col_data::ColData, protocol_result::ProtocolResult},
    data::flights::Flight,
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
    incoming_flights: Option<Vec<Flight>>,

    /// Los vuelso salientes actualmente en memoria.
    departing_flights: Option<Vec<Flight>>,

    /// El tiempo que pasó desde la última _query_.
    last_checked: Instant,

    /// Fecha seleccionada.
    date: DateTime<Local>,

    /// Hilo hijo para cargar vuelos entrantes.
    incoming_child: DateChild,

    /// Extremo de canal que recibe actualizaciones de los vuelos entrantes.
    incoming_receiver: Receiver<Vec<Flight>>,

    /// Hilo hijo para cargar vuelos salientes.
    departing_child: DateChild,

    /// Extremo de canal que recibe actualizaciones de los vuelos entrantes.
    departing_receiver: Receiver<Vec<Flight>>,
}

impl FlightsLoader {
    /// Crea una nueva instancia de cargador de vuelos.
    pub fn new(
        client: Arc<Mutex<Client>>,
        flights: (Option<Vec<Flight>>, Option<Vec<Flight>>),
        last_checked: Instant,
        date: DateTime<Local>,
        children: (DateChild, DateChild),
        receivers: (Receiver<Vec<Flight>>, Receiver<Vec<Flight>>),
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
        to_parent: Sender<Vec<Flight>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_child(to_parent, LoadableFlight::Incoming, client)
    }

    /// Genera el hilo hijo para cargar vuelos salientes.
    pub fn gen_departing_child(
        to_parent: Sender<Vec<Flight>>,
        client: Arc<Mutex<Client>>,
    ) -> DateChild {
        Self::gen_date_child(to_parent, LoadableFlight::Departing, client)
    }

    /// Genera un hilo hijo.
    fn gen_date_child(
        to_parent: Sender<Vec<Flight>>,
        flight_type: LoadableFlight,
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

                        let query = match flight_type {
                            LoadableFlight::Incoming => format!(
                                "SELECT * FROM vuelos_entrantes WHERE llegada < {} AND llegada > {};",
                                timestamp + (DAY_IN_SECONDS / 2),
                                timestamp - (DAY_IN_SECONDS / 2),
                            ),
                            LoadableFlight::Departing => format!(
                                "SELECT * FROM vuelos_salientes WHERE salida < {} AND salida > {};",
                                timestamp + (DAY_IN_SECONDS / 2),
                                timestamp - (DAY_IN_SECONDS / 2),
                            ),
                        };

                        let flights = match Self::load_flights(Arc::clone(&client), query.as_str())
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
    fn load_flights(client_lock: Arc<Mutex<Client>>, query: &str) -> Result<Vec<Flight>> {
        let mut client = match client_lock.lock() {
            Err(poison_err) => {
                return Err(Error::ServerError(format!(
                    "Error de lock envenenado al cargar vuelos:\n\n{}",
                    poison_err
                )));
            }
            Ok(cli) => cli,
        };

        let mut flights = Vec::<Flight>::new();
        let mut tcp_stream = client.connect()?;
        let protocol_result = client.send_query(query, &mut tcp_stream)?;

        // println!("{:?}", protocol_result);

        if let ProtocolResult::Rows(rows) = protocol_result {
            for row in rows {
                if row.len() != 3 {
                    continue;
                }

                // 1. Origen.
                if let ColData::String(orig) = &row[0] {
                    // 2. Destino.
                    if let ColData::String(dest) = &row[1] {
                        // 3. Fecha.
                        if let ColData::String(timestamp) = &row[2] {
                            let id = timestamp.parse::<i32>().unwrap_or(0);
                            let true_timestamp = timestamp.parse::<Long>().unwrap_or(0);
                            flights.push(Flight::new(
                                id,
                                orig.to_string(),
                                dest.to_string(),
                                true_timestamp,
                            ));
                        }
                    }
                }
            }
        } else if let ProtocolResult::QueryError(err) = protocol_result {
            println!("{}", err);
        }

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
        let (incoming_sender, incoming_receiver) = channel::<Vec<Flight>>();
        let (departing_sender, departing_receiver) = channel::<Vec<Flight>>();

        let incoming_client = Arc::clone(&client);
        let departing_client = Arc::clone(&client);

        Self::new(
            client,
            (Some(Vec::<Flight>::new()), Some(Vec::<Flight>::new())),
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
                self.incoming_flights = Some(new_incoming);
            }
        }

        if let Ok(new_departing) = self.departing_receiver.try_recv() {
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
