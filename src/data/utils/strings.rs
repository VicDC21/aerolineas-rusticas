//! Módulo para funciones auxiliares de [String]s.

/// Saca el caracter **LF** (`"\n"`), o de ser posible **CRLF** (`"\r\n"`).
///
/// ```rust
/// use aerolineas::data::utils::strings::trim_newline;
///
/// assert_eq!("hola\tmundo!".to_string(), trim_newline("hola\tmundo!\r\n"))
/// ```
pub fn trim_newline(string: &str) -> String {
    string.trim_ascii_end().to_string()
}

/// Saca comillas dobles (`"`).
///
/// ```rust
/// use aerolineas::data::utils::strings::trim_quotes;
///
/// assert_eq!("hola\tmundo!".to_string(), trim_quotes("\"hola\tmundo!\""))
/// ```
pub fn trim_quotes(string: &str) -> String {
    string
        .trim_start_matches('"')
        .trim_end_matches('"')
        .to_string()
}

/// Combina las operaciones de [trim_newline()] y [trim_quotes()].
///
/// ```rust
/// use aerolineas::data::utils::strings::sanitize;
///
/// assert_eq!("hola\tmundo!".to_string(), sanitize("\"hola\tmundo!\"\r\n"))
/// ```
pub fn sanitize(string: &str) -> String {
    let quote = '"';
    string
        .trim_ascii_end()
        .trim_start_matches(quote)
        .trim_end_matches(quote)
        .to_string()
}

/// [Sanitiza](sanitize()) un [String] y lo descompone en partes.
///
/// ```rust
/// use aerolineas::data::utils::strings::breakdown;
///
/// assert_eq!(vec!["hola".to_string(), "mundo".to_string(), "!".to_string()], breakdown("\"hola,mundo,!\"\r\t\n", ','));
/// ```
pub fn breakdown(string: &str, delimiter: char) -> Vec<String> {
    sanitize(string)
        .split(delimiter)
        .map(|elem| elem.to_string())
        .collect::<Vec<String>>()
}

/// Convierte un &[str] a un [Option] dependiendo de si está vacío o no.
/// 
/// * Si el &[str] es `""`, entonces se devuelve [Option::None].
/// * Cualquier otro caso devuelve [Option::Some]\([str].to_string())
/// 
/// ```rust
/// use aerolineas::data::utils::strings::to_option;
/// 
/// let empty = to_option("");
/// let example = to_option("hola mundo!");
///
/// assert!(matches!(empty, None));
/// assert!(matches!(example, Some(_)));
/// if let Some(ex) = example {
///     assert_eq!("hola mundo!".to_string(), ex);
/// }
/// ```
pub fn to_option(string: &str) -> Option<String> {
    match string {
        "" => None,
        _ => Some(string.to_string())
    }
}
