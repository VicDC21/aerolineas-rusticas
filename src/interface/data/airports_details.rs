//! Módulo para estructura que guarda detalles de plugins de aeropuertos en la interfaz.

use std::sync::Arc;

use crate::data::{airports::Airport, flights::Flight};

/// Holds details to many of the important data of the current instant in the GUI.
pub struct AirportsDetails {
    /// El puerto seleccionado actualmente.
    selected_airport: Option<Airport>,

    /// Un aeropuerto extra, para acciones especiales.
    extra_airport: Option<Airport>,

    /// Los aeropuertos actualmente en memoria.
    current_airports: Arc<Vec<Airport>>,

    /// Los vuelos actualmente en memoria.
    current_flights: Arc<Vec<Flight>>,

    /// Decide si mostrar ciertos detalles o no.
    show_airports_list: bool,
}

impl AirportsDetails {
    /// Crea una nueva instancia.
    pub fn new(
        selected_airport: Option<Airport>,
        extra_airport: Option<Airport>,
        current_airports: Vec<Airport>,
        current_flights: Vec<Flight>,
        show_airports_list: bool,
    ) -> Self {
        Self {
            selected_airport,
            extra_airport,
            current_airports: Arc::new(current_airports),
            current_flights: Arc::new(current_flights),
            show_airports_list,
        }
    }

    /// Consigue una referencia al aeropuerto principal actualmente seleccionado.
    pub fn get_selected_airport(&self) -> &Option<Airport> {
        &self.selected_airport
    }

    /// Actualiza el aeropuerto principal.
    pub fn set_selected_airport(&mut self, new_selection: Option<Airport>) {
        self.selected_airport = new_selection;
    }

    /// Consigue una referencia al aeropuerto secundario actualmente seleccionado.
    pub fn get_extra_airport(&self) -> &Option<Airport> {
        &self.extra_airport
    }

    /// Actualiza el aeropuerto secundario.
    pub fn set_extra_airport(&mut self, new_extra: Option<Airport>) {
        if self.selected_airport.is_none()
            || Self::same_airports(&self.selected_airport, &new_extra)
        {
            self.extra_airport = None;
        } else {
            self.extra_airport = new_extra;
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

    /// Consigue una referencia clonada a los vuelos guardados.
    pub fn get_flights(&self) -> Arc<Vec<Flight>> {
        Arc::clone(&self.current_flights)
    }

    /// Actualiza la lista de vuelos.
    pub fn set_flights(&mut self, new_flights: Vec<Flight>) {
        if !new_flights.is_empty() {
            self.current_flights = Arc::new(new_flights);
        }
    }

    /// Consigue si mostrar la lista de aeropuertos o no.
    pub fn get_show_airports_list(&self) -> &bool {
        &self.show_airports_list
    }

    /// Decide si mostrar la lista de aeropuertos o no.
    pub fn set_show_airports_list(&mut self, do_show_airports: bool) {
        self.show_airports_list = do_show_airports;
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

impl Default for AirportsDetails {
    fn default() -> Self {
        Self::new(
            None,
            None,
            Vec::<Airport>::new(),
            Vec::<Flight>::new(),
            false,
        )
    }
}
