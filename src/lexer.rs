pub use self::token::Token {
    Def,
    Extern,
    Delimiter,
    OpeningParenthesis,
    ClosingParenthesis,
    Comma,
    Identifier(String),
    Number(f64),
    Operator(String)
};
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Def,
    Extern,
    Delimiter,
    OpeningParenthesis,
    ClosingParenthesis,
    Comma,
    Identifier(String),
    Number(f64),
    Operator(String)
}

pub fn tokenize(input: &str) -> Vec<Token> {

    let comment_regex = Regex::new(r"//.*").unwrap();
    let preprocessed = comment.replace_all(input, "\n");

    let mut result = Vec::new();

    let token_regex = regex!(concat!(
        r"(?P<identifier>[a-zA-Z][a-zA-Z0-9]*)|",
        r"(?P<number>[0-9]+(\.[0-9]+)?)|",
        r"(?P<operator>\S)|",
        r"(?P<delimiter>\(|\)|,)",
        r"(?P<oppar>\()|",
        r"(?P<clpar>\))|",
        r"(?P<comma>,)"
    ));

    for cap in token.captures_iter(preprocessed.as_str()) {
        let token = if let Some(ident) = cap.name("ident") {
            match ident.as_str() {
                "def" => Token::Def,
                "extern" => Token::Extern,
                identifier => Token::Identifier(identifier.to_string())
            }
        } else if let Some(number) = cap.name("number") {
            Token::Number(number.as_str().parse().unwrap())
        } else if let Some(operator) = cap.name("operator") {
            Token::Operator(operator.as_str().to_string())
        } else if let Some(delimiter) = cap.name("delimiter") {
            Token::Delimiter
        } else if let Some(oppar) = cap.name("oppar") {
            Token::OpeningParenthesis
        } else if let Some(clpar) = cap.name("clpar") {
            Token::ClosingParenthesis
        } else if let Some(comma) = cap.name("comma") {
            Token::Comma
        } else {
            panic!("Unrecognized token: {}", cap.at(0).unwrap());
        };
        result.push(token);
    }
    result
}