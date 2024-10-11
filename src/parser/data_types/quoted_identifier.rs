/// '"' (any character where " can appear if doubled)+ '"'

pub struct QuotedIdentifier {
    text: String,
}


impl QuotedIdentifier{
    pub fn new(text: String) -> Self{
        QuotedIdentifier{
            text
        }
    }

    pub fn check_quoted_identifier(first: &String, second: &String, third: &String) -> bool{
        if first != "\"" || third != "\""{
            return false
        }
        if !second.chars().all(char::is_alphabetic){
            return false
        }
        true
    }
}