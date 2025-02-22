//! Módulo para contener los distintos tipos de aeropuertos.
use {
    crate::traits::PrettyShow,
    protocol::errors::error::Error,
    std::{
        convert::TryFrom,
        fmt::{Display, Formatter, Result as FmtResult},
    },
};

/// El tipo de un aeropuerto.
///
/// Las definiciones se inspiran en las de [OurAirports](https://ourairports.com/help/#airports).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AirportType {
    /// Un gran aeropuerto con un tráfico anual en millones de personas, o una base militar importante.
    LargeAirport,

    /// Un aeropuerto con servicio regional, aviación general o tráfico militar.
    MediumAirport,

    /// Un aeropuerto con servicio ligero o casi sin actividad.
    SmallAirport,

    /// Puestos para helicópteros, sin pistas disponibles sino para descenso vertical.
    Heliport,

    /// Puerto para hidroaviones, sin pistas de aterrizaje para aviones convencionales.
    SeaplaneBase,

    /// Área para lanzar globos de aire caliente.
    BalloonBase,

    /// Cualquier tipo de aeropuerto que no es actualmente operacional.
    Closed,
}

impl PrettyShow for AirportType {
    fn pretty_name(&self) -> &str {
        match self {
            Self::LargeAirport => "Large Airport",
            Self::MediumAirport => "Medium Airport",
            Self::SmallAirport => "Small Airport",
            Self::Heliport => "Heliport",
            Self::SeaplaneBase => "Seaplane Base",
            Self::BalloonBase => "Ballon Port",
            Self::Closed => "Closed",
        }
    }
}

impl Display for AirportType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::LargeAirport => write!(f, "large_airport"),
            Self::MediumAirport => write!(f, "medium_airport"),
            Self::SmallAirport => write!(f, "small_airport"),
            Self::Heliport => write!(f, "heliport"),
            Self::SeaplaneBase => write!(f, "seaplane_base"),
            Self::BalloonBase => write!(f, "balloonport"), // Sí, sin `_`
            Self::Closed => write!(f, "closed"),
        }
    }
}

impl TryFrom<&str> for AirportType {
    type Error = Error;
    fn try_from(airport_type: &str) -> Result<Self, Self::Error> {
        match airport_type {
            "large_airport" => Ok(Self::LargeAirport),
            "medium_airport" => Ok(Self::MediumAirport),
            "small_airport" => Ok(Self::SmallAirport),
            "heliport" => Ok(Self::Heliport),
            "seaplane_base" => Ok(Self::SeaplaneBase),
            "balloonport" => Ok(Self::BalloonBase),
            "closed" => Ok(Self::Closed),
            _ => Err(Error::ServerError(format!(
                "'{}' no es un tipo válido de aeropuerto.",
                airport_type
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_nombres_correctos() {
        let types = [
            AirportType::LargeAirport,
            AirportType::MediumAirport,
            AirportType::SmallAirport,
            AirportType::Heliport,
            AirportType::SeaplaneBase,
            AirportType::BalloonBase,
            AirportType::Closed,
        ];
        let expected = [
            "large_airport",
            "medium_airport",
            "small_airport",
            "heliport",
            "seaplane_base",
            "balloonport",
            "closed",
        ];

        for i in 0..types.len() {
            assert_eq!(types[i].to_string(), expected[i].to_string());
        }
    }

    #[test]
    fn test_2_crea_buenas_instancias() {
        let large_res = AirportType::try_from("large_airport");
        let medium_res = AirportType::try_from("medium_airport");
        let small_res = AirportType::try_from("small_airport");
        let heli_res = AirportType::try_from("heliport");
        let seaplane_res = AirportType::try_from("seaplane_base");
        let balloon_res = AirportType::try_from("balloonport");
        let closed_res = AirportType::try_from("closed");

        assert!(large_res.is_ok());
        if let Ok(large) = large_res {
            assert!(matches!(large, AirportType::LargeAirport));
        }

        assert!(medium_res.is_ok());
        if let Ok(medium) = medium_res {
            assert!(matches!(medium, AirportType::MediumAirport));
        }

        assert!(small_res.is_ok());
        if let Ok(small) = small_res {
            assert!(matches!(small, AirportType::SmallAirport));
        }

        assert!(heli_res.is_ok());
        if let Ok(heli) = heli_res {
            assert!(matches!(heli, AirportType::Heliport));
        }

        assert!(seaplane_res.is_ok());
        if let Ok(seaplane) = seaplane_res {
            assert!(matches!(seaplane, AirportType::SeaplaneBase));
        }

        assert!(balloon_res.is_ok());
        if let Ok(balloon) = balloon_res {
            assert!(matches!(balloon, AirportType::BalloonBase));
        }

        assert!(closed_res.is_ok());
        if let Ok(closed) = closed_res {
            assert!(matches!(closed, AirportType::Closed));
        }
    }

    #[test]
    fn test_3_nombre_incorrecto() {
        let mal = AirportType::try_from("pepperoni");

        assert!(mal.is_err());
        if let Err(err) = mal {
            assert!(matches!(err, Error::ServerError(_)));
        }
    }
}
