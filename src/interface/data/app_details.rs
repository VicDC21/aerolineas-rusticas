//! Módulo para estructura que guarda detalles de plugins de aeropuertos en la interfaz.

use std::sync::Arc;

use crate::data::{
    airports::airp::Airport,
    flights::{departing::DepartingFlight, incoming::IncomingFlight},
};

/// Holds details to many of the important data of the current instant in the GUI.
pub struct AirlinesDetails {
    /// El puerto seleccionado actualmente.
    selected_airport: Arc<Option<Airport>>,

    /// Un aeropuerto extra, para acciones especiales.
    extra_airport: Arc<Option<Airport>>,

    /// Los aeropuertos actualmente en memoria.
    current_airports: Arc<Vec<Airport>>,

    /// Los vuelos entrantes actualmente en memoria.
    incoming_flights: Arc<Vec<IncomingFlight>>,

    /// Los vuelos salientes actualmente en memoria.
    departing_flights: Arc<Vec<DepartingFlight>>,

    /// Decidir si mostrar los vuelos entrantes.
    show_incoming_flights: bool,

    /// Decide si mostrar los vuelos salientes.
    show_departing_flights: bool,
}

impl AirlinesDetails {
    /// Crea una nueva instancia.
    pub fn new(
        selected_airport: Option<Airport>,
        extra_airport: Option<Airport>,
        current_airports: Vec<Airport>,
        incoming_flights: Vec<IncomingFlight>,
        departing_flights: Vec<DepartingFlight>,
        show_incoming_flights: bool,
        show_departing_flights: bool,
    ) -> Self {
        Self {
            selected_airport: Arc::new(selected_airport),
            extra_airport: Arc::new(extra_airport),
            current_airports: Arc::new(current_airports),
            incoming_flights: Arc::new(incoming_flights),
            departing_flights: Arc::new(departing_flights),
            show_incoming_flights,
            show_departing_flights,
        }
    }

    /// Consigue una referencia al aeropuerto principal actualmente seleccionado.
    pub fn get_ref_selected_airport(&self) -> &Option<Airport> {
        self.selected_airport.as_ref()
    }

    /// Consigue una referencia clonada al aeropuerto principal actualmente seleccionado.
    pub fn get_selected_airport(&self) -> Arc<Option<Airport>> {
        Arc::clone(&self.selected_airport)
    }

    /// Actualiza el aeropuerto principal.
    pub fn set_selected_airport(&mut self, new_selection: Option<Airport>) {
        // Para evitar que un aeropuerto seleccionado muestre los vuelos de otro
        if new_selection.is_some() || !Self::same_airports(&self.selected_airport, &new_selection) {
            self.set_incoming_flights(Vec::<IncomingFlight>::new(), true);
            self.set_departing_flights(Vec::<DepartingFlight>::new(), true);
        }

        if new_selection.is_none() || Self::same_airports(&self.extra_airport, &new_selection) {
            self.selected_airport = Arc::new(None);
            self.extra_airport = Arc::new(None);
        } else {
            self.selected_airport = Arc::new(new_selection);
        }
    }

    /// Consigue una referencia al aeropuerto secundario actualmente seleccionado.
    pub fn get_ref_extra_airport(&self) -> &Option<Airport> {
        self.extra_airport.as_ref()
    }

    /// Actualiza el aeropuerto secundario.
    pub fn set_extra_airport(&mut self, new_extra: Option<Airport>) {
        if self.selected_airport.is_none()
            || Self::same_airports(&self.selected_airport, &new_extra)
        {
            self.extra_airport = Arc::new(None);
        } else {
            self.extra_airport = Arc::new(new_extra);
        }
    }

    /// Consigue una referencia clonada a los aeropuertos guardados.
    pub fn get_airports(&self) -> Arc<Vec<Airport>> {
        Arc::clone(&self.current_airports)
    }

    /// Actualiza la lista de aeropuertos.
    pub fn set_airports(&mut self, new_airports: Vec<Airport>) {
        self.current_airports = Arc::new(new_airports);
    }

    /// Consigue una referencia clonada a los vuelos entrantes guardados.
    pub fn get_incoming_flights(&self) -> Arc<Vec<IncomingFlight>> {
        Arc::clone(&self.incoming_flights)
    }

    /// Actualiza la lista de vuelos entrantes.
    pub fn set_incoming_flights(&mut self, new_incoming: Vec<IncomingFlight>, ignore_empty: bool) {
        if ignore_empty || !new_incoming.is_empty() {
            self.incoming_flights = Arc::new(new_incoming);
        }
    }

    /// Consigue una referencia clonada a los vuelos salientes guardados.
    pub fn get_departing_flights(&self) -> Arc<Vec<DepartingFlight>> {
        Arc::clone(&self.departing_flights)
    }

    /// Actualiza la lista de vuelos entrantes.
    pub fn set_departing_flights(
        &mut self,
        new_departing: Vec<DepartingFlight>,
        ignore_empty: bool,
    ) {
        if ignore_empty || !new_departing.is_empty() {
            self.departing_flights = Arc::new(new_departing);
        }
    }

    /// Consigue si mostrar la lista de vuelos entrantes o no.
    pub fn get_show_incoming_flights(&self) -> &bool {
        &self.show_incoming_flights
    }

    /// Decide si mostrar la lista de vuelos entrantes o no.
    pub fn set_show_incoming_flights(&mut self, do_show_incoming: bool) {
        self.show_incoming_flights = do_show_incoming;
    }

    /// Consigue si mostrar la lista de vuelos salientes o no.
    pub fn get_show_departing_flights(&self) -> &bool {
        &self.show_departing_flights
    }

    /// Decide si mostrar la lista de vuelos salientes o no.
    pub fn set_show_departing_flights(&mut self, do_show_departing: bool) {
        self.show_departing_flights = do_show_departing;
    }

    /// Verifica si dos opciones de aeropuertos son los mismos.
    fn same_airports(airport_1_opt: &Option<Airport>, airport_2_opt: &Option<Airport>) -> bool {
        if let Some(airport_1) = airport_1_opt {
            if let Some(airport_2) = airport_2_opt {
                if airport_1 == airport_2 {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for AirlinesDetails {
    fn default() -> Self {
        Self::new(
            None,
            None,
            Vec::<Airport>::new(),
            Vec::<IncomingFlight>::new(),
            Vec::<DepartingFlight>::new(),
            false,
            false,
        )
    }
}
