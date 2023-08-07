use crate::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub enum ExprAST {
    Number(f64),
    Variable(String),
    BinaryOp(char, Box<ExprAST>, Box<ExprAST>),
    Call(String, Vec<ExprAST>),
    If {
        condition: Box<ExprAST>,
        then: Box<ExprAST>,
        else_: Box<ExprAST>,
    },
    For {
        variable_name: String,
        start: Box<ExprAST>,
        end: Box<ExprAST>,
        step: Box<ExprAST>,
        body: Box<ExprAST>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrototypeAST(pub String, pub Vec<String>);

#[derive(Debug, PartialEq)]
pub struct FunctionAST(pub PrototypeAST, pub ExprAST);

type ParseResult<T> = Result<T, String>;

pub struct Parser<I>
    where I: Iterator<Item=char>
{
    lexer: I,
    current_token: Option<token>,
}

impl<I> Parser<I>
    where I: Iterator<Item=char>,
{
    pub fn new(lexer: Lexer<I>) -> Self {
        Parser {
            lexer: lexer,
            current_token: None
        }
    }

    pub fn current_token(&self) -> &Token {
        self.current_token.as_ref().expect("Parser: No current token")
    }

    pub fn get_next_token(&mut self) {
        self.current_token = self.lexer.next();
        self.current_token.clone()
    }

    fn parse_number(&mut self) -> ParseResult<ExprAST> {
        match *self.current_token() {
            Token::Number(value) => {
                self.get_next_token();
                Ok(ExprAST::Number(value))
            }
            ref token => Err(format!("Expected number, found {:?}", token)),
        }
    }

    fn parse_paren_expr(&mut self) -> ParseResult<ExprAST> {
        assert_eq!(*self.current_token(), Token::Char('('));
        self.get_next_token();

        let expr = self.parse_expression()?;
        if *self.current_token() != Token::Char(')') {
            return Err(format!("Expected ')', found {:?}", self.current_token()));
        }
        self.get_next_token();
        Ok(expr)
    }

    fn parse_identifier_expr(&mut self) -> ParseResult<ExprAST> {
        let identifier = match *self.current_token() {
            Token::Identifier(ref name) => name.clone(),
            ref token => return Err(format!("Expected identifier, found {:?}", token)),
        };
        self.get_next_token();

        if *self.current_token() != Token::Char('(') {
            return Ok(ExprAST::Variable(identifier));
        }

        self.get_next_token();
        let mut args = Vec::new();
        if *self.current_token() != Token::Char(')') {
            loop {
                args.push(self.parse_expression()?);
                if *self.current_token() == Token::Char(')') {
                    break;
                }
                if *self.current_token() != Token::Char(',') {
                    return Err(format!("Expected ')' or ',', found {:?}", self.current_token()));
                }
                self.get_next_token();
            }
        }
        self.get_next_token();
        Ok(ExprAST::Call(identifier, args))
    }

    fn parse_if_expr(&mut self) -> ParseResult<ExprAST> {
        assert_eq!(*self.current_token(), Token::If);
        self.get_next_token();

        let condition = self.parse_expression()?;
        if *self.current_token() != Token::Then {
            return Err(format!("Expected 'then', found {:?}", self.current_token()));
        }
        self.get_next_token();

        let then = self.parse_expression()?;
        if *self.current_token() != Token::Else {
            return Err(format!("Expected 'else', found {:?}", self.current_token()));
        }
        self.get_next_token();

        let else_ = self.parse_expression()?;
        Ok(ExprAST::If {
            condition: Box::new(condition),
            then: Box::new(then),
            else_: Box::new(else_),
        })
    }

    fn parse_for_expr(&mut self) -> ParseResult<ExprAST> {
        assert_eq!(*self.current_token(), Token::For);
        self.get_next_token();

        let variable_name = match *self.current_token() {
            Token::Identifier(ref name) => name.clone(),
            ref token => return Err(format!("Expected identifier, found {:?}", token)),
        };
        self.get_next_token();

        if *self.current_token() != Token::Char('=') {
            return Err(format!("Expected '=', found {:?}", self.current_token()));
        }
        self.get_next_token();

        let start = self.parse_expression()?;
        if *self.current_token() != Token::Char(',') {
            return Err(format!("Expected ',', found {:?}", self.current_token()));
        }
        self.get_next_token();

        let end = self.parse_expression()?;
        let step = if *self.current_token() == Token::Char(',') {
            self.get_next_token();
            Some(self.parse_expression()?)
        } else {
            None
        };

        if *self.current_token() != Token::In {
            return Err(format!("Expected 'in', found {:?}", self.current_token()));
        }
        self.get_next_token();

        let body = self.parse_expression()?;
        Ok(ExprAST::For {
            variable_name: variable_name,
            start: Box::new(start),
            end: Box::new(end),
            step: Box::new(step),
            body: Box::new(body),
        })
    }

    fn parse_expression(&mut self) -> ParseResult<ExprAST> {
        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }

    fn parse_bin_op_rhs(&mut self, expr_prec: u8, lhs: ExprAST) -> ParseResult<ExprAST> {
        let mut lhs = lhs;
        loop {
            let token_prec = get_token_precedence(self.current_token());
            if token_prec < expr_prec {
                return Ok(lhs);
            }
            let bin_op = match *self.current_token() {
                Token::Char(c) => c,
                ref token => return Err(format!("Expected operator, found {:?}", token)),
            };
            self.get_next_token();
            let mut rhs = self.parse_primary()?;
            let next_prec = get_token_precedence(self.current_token());
            if token_prec < next_prec {
                rhs = self.parse_bin_op_rhs(token_prec + 1, rhs)?;
            }
            lhs = ExprAST::BinaryOp(bin_op, Box::new(lhs), Box::new(rhs));
        }
    }

    fn parse_primary(&mut self) -> ParseResult<ExprAST> {
        match *self.current_token() {
            Token::Identifier(_) => self.parse_identifier_expr(),
            Token::Number(_) => self.parse_num_expr(),
            Token::Char('(') => self.parse_paren_expr(),
            Token::If => self.parse_if_expr(),
            Token::For => self.parse_for_expr(),
            _ => Err(format!("Expected primary expression, found {:?}", self.current_token())),
        }
    }

    fn parse_prototype(&mut self) -> ParseResult<PrototypeAST> {
        let name = match *self.current_token() {
            Token::Identifier(ref name) => name.clone(),
            ref token => return Err(format!("Expected identifier, found {:?}", token)),
        };
        self.get_next_token();

        if *self.current_token() != Token::Char('(') {
            return Err(format!("Expected '(', found {:?}", self.current_token()));
        }
        self.get_next_token();

        let mut args = Vec::new();
        while let Token::Identifier(ref name) = *self.current_token() {
            args.push(name.clone());
            self.get_next_token();
        }

        if *self.current_token() != Token::Char(')') {
            return Err(format!("Expected ')', found {:?}", self.current_token()));
        }
        self.get_next_token();

        Ok(PrototypeAST(name, args))
    }

    pub fn parse_definition(&mut self) -> ParseResult<FunctionAST> {
        assert_eq!(*self.current_token(), Token::Def);
        self.get_next_token();

        let proto = self.parse_prototype()?;
        let body = self.parse_expression()?;
        Ok(FunctionAST(proto, body))
    }

    pub fn parse_external(&mut self) -> ParseResult<PrototypeAST> {
        assert_eq!(*self.current_token(), Token::Extern);
        self.get_next_token();
        self.parse_prototype()
    }

    pub fn parse_top_level_expr(&mut self) -> ParseResult<FunctionAST> {
        let proto = PrototypeAST("".to_string(), Vec::new());
        Ok(FunctionAST(proto, self.parse_expression()?))
    }
}

fn get_token_precedence(token: &Token) -> u8 {
    match *token {
        Token::Char(c) => match c {
                '<' => 10,
                '+' => 20,
                '-' => 20,
                '*' => 40,
                _ => -1,
        },
        _ => -1,
    }
}