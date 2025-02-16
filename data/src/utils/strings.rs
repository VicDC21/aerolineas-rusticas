//! Módulo para funciones auxiliares de [String]s.

use crate::aliases::results::Result;

/// Saca el caracter **LF** (`"\n"`), o de ser posible **CRLF** (`"\r\n"`).
///
/// ```rust
/// use data::utils::strings::trim_newline;
///
/// assert_eq!("hola\tmundo!".to_string(), trim_newline("hola\tmundo!\r\n"))
/// ```
pub fn trim_newline(string: &str) -> String {
    string.trim_ascii_end().to_string()
}

/// Saca comillas dobles (`"`).
///
/// ```rust
/// use data::utils::strings::trim_quotes;
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
/// use data::utils::strings::sanitize;
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
/// use data::utils::strings::breakdown;
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
/// use data::utils::strings::to_option;
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
        _ => Some(string.to_string()),
    }
}

/// Une strings que empiezan y terminan en comillas en uno sólo.
///
/// ```rust
/// use data::utils::strings::unify_quotes_tokens;
///
/// let first_res = unify_quotes_tokens(vec!["a", "\"b", "c", "d\"", "e"]);
/// assert!(first_res.is_ok());
/// if let Ok(first) = first_res {
///     assert_eq!(first, vec!["a".to_string(), "bcd".to_string(), "e".to_string()]);
/// }
/// ```
pub fn unify_quotes_tokens(tokens: Vec<&str>) -> Result<Vec<String>> {
    let mut new_tokens = Vec::<String>::new();
    let mut buffer = Vec::<&str>::new();

    for raw in tokens {
        let token = raw.trim_ascii();
        let mut comillas = false;

        if token.starts_with("\"") && token != "\"" {
            comillas = true;
            // si se encuentra una comilla que abre antes de encontrar una que cierra,
            // agregar el buffer como si fueran elementos comunes, y NO fusionarlos.
            if !buffer.is_empty() {
                new_tokens.extend(buffer.iter().map(|elem| sanitize(elem)));
                buffer.clear();
            }
            buffer.push(token);
        }
        if token.ends_with("\"") {
            comillas = true;
            // pushear al buffer salvo que se trate del mismo elemento que también tiene comillas que abren
            if !(buffer.len() == 1 && buffer[0].ends_with("\"")) {
                buffer.push(token);
            }
            new_tokens.push(sanitize(&buffer.join("")));
            buffer.clear();
        }
        if !comillas {
            if buffer.is_empty() {
                // a este punto, el token no tiene comillas ni al principio ni al final
                new_tokens.push(sanitize(token));
            } else {
                buffer.push(token);
            }
        }
    }
    if !buffer.is_empty() {
        new_tokens.extend(buffer.iter().map(|elem| sanitize(elem)));
    }
    Ok(new_tokens)
}
