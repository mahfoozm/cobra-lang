pub use self::token::Token {
    Def,
    Extern,
    Delimiter,
    OpeningParenthesis,
    ClosingParenthesis,
    Comma,
    Identifier(String),
    Number(f64),
    Operator(String),
    If,
    Then,
    Else,
    For,
    In,
};

pub struct Lexer<I>
    where I: Iterator<Item=char>
{
    input: Peekable<I>,
    last_char: Option<char>,
}

impl<I> Lexer<I>
    where I: Iterator<Item=char>,
{
    pub fn new(mut input: I) -> Lexer<I> {
        let last_char = input.next();
        Lexer {
            input: input.peekable(),
            last_char: last_char,
        }
    }

    fn step(&mut self) -> Option<char> {
        self.last_char = self.input.next();
        self.last_char
    }

    pub fn gettok(&mut self) -> Token {
        while matches!(self.last_char, Some(' ') | Some('\t') | Some('\n')) {
            self.step();
        }

        let last_char = if let Some(c) = self.last_char {
            c
        } else {
            return Token::Eof;
        };

        if last_char.is_ascii_alphabetic() {
            let mut identifier = String::new();
            identifier.push(last_char);
            
            while let Some(&c) = self.input.peek() {
                if c.is_ascii_alphanumeric() {
                    identifier.push(c);
                    self.step();
                } else {
                    break;
                }
            }

            match identifier.as_str() {
                "def" => Token::Def,
                "extern" => Token::Extern,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "for" => Token::For,
                "in" => Token::In,
                _ => Token::Identifier(identifier),
            }

        } else if last_char.is_ascii_digit() || last_char == '.' {
            let mut number = String::new();
            number.push(last_char);
            while let Some(&c) = self.input.peek() {
                if c.is_ascii_digit() || c == '.' {
                    number.push(c);
                    self.step();
                } else {
                    break;
                }
            }
            Token::Number(number.parse().expect("Lexer: Invalid number"))

        } else if last_char == '#' {
            while let Some(&c) = self.input.peek() {
                if c != '\n' {
                    self.step();
                } else {
                    break;
                }
            }
            self.gettok()

        } else {
            let mut operator = String::new();
            operator.push(last_char);
            while let Some(&c) = self.input.peek() {
                if !c.is_ascii_alphanumeric() {
                    operator.push(c);
                    self.step();
                } else {
                    break;
                }
            }

            Token::Operator(operator)
        }
    }
}