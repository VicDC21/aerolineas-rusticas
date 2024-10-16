pub fn tokenize_query(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    for c in query.chars() {
        match c {
            '\'' => handle_single_quote(
                &mut tokens,
                &mut current_token,
                &mut in_single_quotes,
                &mut in_double_quotes,
            ),
            '"' => handle_double_quote(&mut tokens, &mut current_token, &mut in_double_quotes),
            ' ' | ',' | ':' | ';' | '(' | ')' | '[' | ']' | '{' | '}' => handle_separator(
                c,
                &mut tokens,
                &mut current_token,
                in_single_quotes,
                in_double_quotes,
            ),
            _ => current_token.push(c),
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }
    tokens
}

fn handle_single_quote(
    tokens: &mut Vec<String>,
    current_token: &mut String,
    in_single_quotes: &mut bool,
    in_double_quotes: &mut bool,
) {
    if !*in_double_quotes {
        push_token_if_not_empty(tokens, current_token);
        tokens.push("'".to_string());
        *in_single_quotes = !*in_single_quotes;
    } else {
        current_token.push('\'');
    }
}

fn handle_double_quote(
    tokens: &mut Vec<String>,
    current_token: &mut String,
    in_double_quotes: &mut bool,
) {
    push_token_if_not_empty(tokens, current_token);
    tokens.push("\"".to_string());
    *in_double_quotes = !*in_double_quotes;
}

fn handle_separator(
    c: char,
    tokens: &mut Vec<String>,
    current_token: &mut String,
    in_single_quotes: bool,
    in_double_quotes: bool,
) {
    if in_single_quotes || in_double_quotes {
        current_token.push(c);
    } else {
        push_token_if_not_empty(tokens, current_token);
        if !c.is_whitespace() {
            tokens.push(c.to_string());
        }
    }
}

fn push_token_if_not_empty(tokens: &mut Vec<String>, current_token: &mut String) {
    if !current_token.is_empty() {
        let new_token = current_token.split_off(0);
        tokens.push(new_token);
    }
}