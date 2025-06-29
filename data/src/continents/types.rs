//! Módulo para lso tipos de continentes.
use {
    crate::traits::PrettyShow,
    protocol::errors::error::Error,
    std::{
        convert::TryFrom,
        fmt::{Display, Formatter, Result as FmtResult},
    },
};

/// El tipo de un continente.
///
/// Las definiciones están inspiradas en la de [OurAirports](https://ourairports.com/help/data-dictionary.html#airports).
#[derive(Clone, Debug, PartialEq)]
pub enum ContinentType {
    /// Continente de África.
    Africa,

    /// Continente de la Antártida.
    Antarctica,

    /// Continente de Asia.
    Asia,

    /// Continente de Europa.
    Europe,

    /// Continente de América del norte.
    NorthAmerica,

    /// Continente de Oceanía.
    Oceania,

    /// Continente de América del Sur.
    SouthAmerica,
}

impl PrettyShow for ContinentType {
    fn pretty_name(&self) -> &str {
        match self {
            Self::Africa => "Africa",
            Self::Antarctica => "Antarctica",
            Self::Asia => "Asia",
            Self::Europe => "Europe",
            Self::NorthAmerica => "North America",
            Self::Oceania => "Oceania",
            Self::SouthAmerica => "South America",
        }
    }
}

impl Display for ContinentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Africa => write!(f, "AF"),
            Self::Antarctica => write!(f, "AN"),
            Self::Asia => write!(f, "AS"),
            Self::Europe => write!(f, "EU"),
            Self::NorthAmerica => write!(f, "NA"),
            Self::Oceania => write!(f, "OC"),
            Self::SouthAmerica => write!(f, "SA"),
        }
    }
}

impl TryFrom<&str> for ContinentType {
    type Error = Error;
    fn try_from(continent: &str) -> Result<Self, Self::Error> {
        match continent {
            "AF" => Ok(Self::Africa),
            "AN" => Ok(Self::Antarctica),
            "AS" => Ok(Self::Asia),
            "EU" => Ok(Self::Europe),
            "NA" => Ok(Self::NorthAmerica),
            "OC" => Ok(Self::Oceania),
            "SA" => Ok(Self::SouthAmerica),
            _ => Err(Error::ServerError(format!(
                "'{continent}' no es un código de continente válido."
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
            ContinentType::Africa,
            ContinentType::Antarctica,
            ContinentType::Asia,
            ContinentType::Europe,
            ContinentType::NorthAmerica,
            ContinentType::Oceania,
            ContinentType::SouthAmerica,
        ];
        let expected = ["AF", "AN", "AS", "EU", "NA", "OC", "SA"];

        for i in 0..types.len() {
            assert_eq!(types[i].to_string(), expected[i].to_string());
        }
    }

    #[test]
    fn test_2_crea_buenas_instancias() {
        let africa_res = ContinentType::try_from("AF");
        let antarctica_res = ContinentType::try_from("AN");
        let asia_res = ContinentType::try_from("AS");
        let europe_res = ContinentType::try_from("EU");
        let north_america_res = ContinentType::try_from("NA");
        let oceania_res = ContinentType::try_from("OC");
        let south_america_res = ContinentType::try_from("SA");

        assert!(africa_res.is_ok());
        if let Ok(africa) = africa_res {
            assert!(matches!(africa, ContinentType::Africa));
        }

        assert!(antarctica_res.is_ok());
        if let Ok(antarctica) = antarctica_res {
            assert!(matches!(antarctica, ContinentType::Antarctica));
        }

        assert!(asia_res.is_ok());
        if let Ok(asia) = asia_res {
            assert!(matches!(asia, ContinentType::Asia));
        }

        assert!(europe_res.is_ok());
        if let Ok(europe) = europe_res {
            assert!(matches!(europe, ContinentType::Europe));
        }

        assert!(north_america_res.is_ok());
        if let Ok(north_america) = north_america_res {
            assert!(matches!(north_america, ContinentType::NorthAmerica));
        }

        assert!(oceania_res.is_ok());
        if let Ok(oceania) = oceania_res {
            assert!(matches!(oceania, ContinentType::Oceania));
        }

        assert!(south_america_res.is_ok());
        if let Ok(south_america) = south_america_res {
            assert!(matches!(south_america, ContinentType::SouthAmerica));
        }
    }

    #[test]
    fn test_3_nombre_incorrecto() {
        let no_existe = ContinentType::try_from("Atlantis");

        assert!(no_existe.is_err());
        if let Err(err) = no_existe {
            assert!(matches!(err, Error::ServerError(_)));
        }
    }
}
