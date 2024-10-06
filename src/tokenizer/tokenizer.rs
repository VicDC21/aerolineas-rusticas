use std::io::{self, Write};

pub fn tokenize_query(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;

    for c in query.chars() {
        match c {
            '"' => handle_quote(&mut tokens, &mut current_token, &mut in_quotes),
            ' ' | ',' | ':' | ';' | '(' | ')' | '[' | ']' | '{'| '}' => handle_separator(c, &mut tokens, &mut current_token, &mut in_quotes),
            _ => current_token.push(c),
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}

fn push_token_if_not_empty(tokens: &mut Vec<String>, current_token: &mut String) {
    if !current_token.is_empty() {
        let new_token = current_token.split_off(0); 
        tokens.push(new_token);
    }
}

fn handle_quote(tokens: &mut Vec<String>, current_token: &mut String, in_quotes: &mut bool) {
    push_token_if_not_empty(tokens, current_token);
    *in_quotes = !*in_quotes;
}

fn handle_separator(c: char, tokens: &mut Vec<String>, current_token: &mut String, in_quotes: &mut bool) {
    if *in_quotes {
        current_token.push(c);
    } else {
        push_token_if_not_empty(tokens, current_token);
        if !c.is_whitespace() {
            tokens.push(c.to_string());
        }
    }
}