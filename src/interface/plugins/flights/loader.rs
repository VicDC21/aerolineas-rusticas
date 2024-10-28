//! Módulo para un cargador de vuelos.

use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::{spawn, JoinHandle},
    time::{Duration, Instant},
};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use eframe::egui::{Painter, Response};
use walkers::{Plugin, Projector};

use crate::{
    client::{cli::Client, col_data::ColData, protocol_result::ProtocolResult},
    data::flights::Flight,
    protocol::aliases::{results::Result, types::Long},
};

/// Un hilo destinado a procesos paralelos.
type ChildHandle = JoinHandle<Result<()>>;

/// El tipo de hilo hijo según fecha.
type DateChild = (Option<ChildHandle>, Sender<(Client, Long)>);

/// Intervalo (en segundos) antes de cargar los vuelos de nuevo, como mínimo.
const FLIGHTS_INTERVAL_SECS: u64 = 5;

/// Un día en segundos.
const DAY_IN_SECONDS: i64 = 86400;

/// Cargador de vuelos.
pub struct FlightsLoader {
    /// El cliente para pedir las queries.
    client: Client,

    /// Los vuelos actualmente ne memoria.
    flights: Option<Vec<Flight>>,

    /// La última vez que [flights](crate::interface::plugins::flights::loader::FlightsLoader::flights)
    /// fue modificado.
    last_checked: Instant,

    /// Fecha seleccionada.
    date: NaiveDate,

    /// Extremo de canal que recibe actualizaciones de los vuelos.
    receiver: Receiver<Vec<Flight>>,

    /// Hilo hijo para cargar vuelos.
    date_child: DateChild,
}

impl FlightsLoader {
    /// Crea una nueva instancia de cargador de vuelos.
    pub fn new(
        client: Client,
        flights: Option<Vec<Flight>>,
        last_checked: Instant,
        date: NaiveDate,
        receiver: Receiver<Vec<Flight>>,
        date_child: DateChild,
    ) -> Self {
        Self {
            client,
            flights,
            last_checked,
            date,
            receiver,
            date_child,
        }
    }

    /// Genera el hilo hijo.
    fn gen_date_child(to_parent: Sender<Vec<Flight>>) -> DateChild {
        let (date_sender, date_receiver) = channel::<(Client, Long)>();
        let date_handle = spawn(move || {
            let stop_value: Long = 0;

            loop {
                match date_receiver.recv() {
                    Ok((client, timestamp)) => {
                        if timestamp == stop_value {
                            break;
                        }

                        let flights = match Self::load_flights(client.clone(), timestamp) {
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

    /// **Consume** la lista de vuelos actualmente en memoria para devolverla, y en su lugar
    /// deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_flights(&mut self) -> Vec<Flight> {
        self.flights.take().unwrap_or_default()
    }

    /// Sincroniza la fecha seleccionada en la aplicación con la guardada aquí.
    pub fn sync_date(&mut self, new_date: NaiveDate) -> &mut Self {
        self.date = new_date;
        self
    }

    /// Sincroniza el cliente.
    pub fn sync_client(&mut self, new_client: Client) -> &mut Self {
        self.client = new_client;
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
    fn load_flights(mut client: Client, timestamp: Long) -> Result<Vec<Flight>> {
        let mut flights = Vec::<Flight>::new();
        let mut tcp_stream = client.connect()?;
        let protocol_result = client.send_query(
            format!(
                "SELECT * FROM flights WHERE timestamp < {} AND timestamp > {}",
                timestamp + DAY_IN_SECONDS,
                timestamp,
            )
            .as_str(),
            &mut tcp_stream,
        )?;

        if let ProtocolResult::Rows(rows) = protocol_result {
            for row in rows {
                if row.len() != 4 {
                    continue;
                }

                // 1. El ID.
                if let ColData::Int(id) = &row[0] {
                    // 2. Origen.
                    if let ColData::String(orig) = &row[1] {
                        // 3. Destino.
                        if let ColData::String(dest) = &row[2] {
                            // 4. Fecha.
                            if let ColData::Timestamp(timestamp) = &row[3] {
                                flights.push(Flight::new(
                                    *id,
                                    orig.to_string(),
                                    dest.to_string(),
                                    *timestamp,
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(flights)
    }

    /// Apaga y espera a todos los hilos hijos.
    pub fn wait_children(&mut self) {
        let (date_child, date_sender) = &mut self.date_child;
        if let Some(hanging) = date_child.take() {
            if date_sender.send((self.client.clone(), 0)).is_err() {
                println!("Error mandando un mensaje para parar hilo de área.")
            }
            if hanging.join().is_err() {
                println!("Error esperando a que un hilo hijo termine.")
            }
        }
    }
}

impl Default for FlightsLoader {
    fn default() -> Self {
        let (main_sender, main_receiver) = channel::<Vec<Flight>>();

        Self::new(
            Client::default(),
            Some(Vec::new()),
            Instant::now(),
            Utc::now().date_naive(),
            main_receiver,
            Self::gen_date_child(main_sender.clone()),
        )
    }
}

impl Plugin for &mut FlightsLoader {
    fn run(&mut self, _response: &Response, _painter: Painter, _projector: &Projector) {
        if self.elapsed_at_least(&Duration::from_secs(FLIGHTS_INTERVAL_SECS)) {
            self.reset_instant();

            if let Some(naive_date) = NaiveTime::from_hms_opt(0, 0, 0) {
                let cur_datetime = NaiveDateTime::new(self.date, naive_date);

                let (_, date_sender) = &mut self.date_child;
                if let Err(err) =
                    date_sender.send((self.client.clone(), cur_datetime.and_utc().timestamp()))
                {
                    println!("Error al enviar timestamp al cargador:\n\n{}", err);
                }
            }
        }

        if let Ok(new_flights) = self.receiver.try_recv() {
            if !new_flights.is_empty() {
                self.flights = Some(new_flights);
            }
        }
    }
}

impl Drop for FlightsLoader {
    fn drop(&mut self) {
        self.wait_children();
    }
}
