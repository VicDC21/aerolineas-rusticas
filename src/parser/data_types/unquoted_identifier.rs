/// re('[a-zA-Z][link:[a-zA-Z0-9]]*')

pub struct UnquotedIdentifier {
    text: String,
}

impl UnquotedIdentifier{
    pub fn new(text: String) -> Self{
        UnquotedIdentifier{
            text
        }
    }

    pub fn check_unquoted_identifier(first: &String) -> bool{
        if !first.chars().all(char::is_alphabetic){
            return false
        }
        true
    }


}