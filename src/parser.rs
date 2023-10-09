use crate::lexer::{Token, Tokenizer, is_identifier, get_id};
use std::collections::HashMap;

pub enum ExprAST {
    NumberExprAST { val: f64 },
    VariableExprAST { name: String },
    BinaryExprAST { op: String, lhs: Box<ExprAST>, rhs: Box<ExprAST> },
    PrototypeAST { name: String, args: Vec<String> },
    FunctionAST { proto: Box<ExprAST>, body: Box<ExprAST> },
    CallExprAST  { callee: String, args: Vec<Box<ExprAST>> },
}

pub struct Parser<'a> {
    lexer: &'a mut Tokenizer<'a>,
    prec: HashMap<String, i32>
}


/// Grammar:
impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Tokenizer<'a>) -> Self {
        let mut default_prec = HashMap::<String, i32>::new();
        default_prec.insert("<".to_string(), 10);
        default_prec.insert("+".to_string(), 20);
        default_prec.insert("-".to_string(), 20);
        default_prec.insert("*".to_string(), 30);
        Self {
           lexer: lexer,
           prec: default_prec
        }
    }

    pub fn parse(&mut self) {
        loop {
            match self.lexer.last_token() {
                Token::Eof => break,
                Token::Def => self.handle_definition(),
                Token::Extern => self.handle_extern(),
                _ => self.handle_top_level_expression(),
            }
        }
    }


    fn handle_top_level_expression(&mut self) {
        self.parse_top_level_expr();
    }

    fn handle_extern(&mut self) {
        self.parse_extern();
    }

    fn handle_definition(&mut self) {
        self.parse_definition();
    }

    fn parse_extern(&mut self) -> Option<Box<ExprAST>> {
        self.lexer.next(); // eat `extern`
        self.parse_prototype()
    }

    fn parse_definition(&mut self) -> Option<Box<ExprAST>> {
        self.lexer.next(); // eat `def`
        let Some(proto) = self.parse_prototype() else {
            return None;
        };

        let Some(body) = self.parse_expression() else {
            return None;
        };
        
        Some(Box::new(ExprAST::FunctionAST { proto, body }))
    }

    pub fn parse_top_level_expr(&mut self) -> Option<Box<ExprAST>> {
        let Some(body) = self.parse_expression() else {
            return None;
        };

        let proto = Box::new(ExprAST::PrototypeAST { name: String::from("__anon_expr"), args: Vec::new() });
        Some(Box::new(ExprAST::FunctionAST { proto, body }))
    }

    /// primary
    ///     ::= parenexpr
    ///     ::= identifierexpr
    ///     ::= numberexpr
    fn parse_primary(&mut self) -> Option<Box<ExprAST>> {
        println!("parse_primary");
        if let Token::Identifier{id} = self.lexer.last_token() {
            if id == "(" {
                println!("goto parse_paren_expr");
                return self.parse_paren_expr();
            } else {
                println!("goto parse_identifier_expr");
                return self.parse_identifier_expr();
            }
        } else if let Token::Number{..} = self.lexer.last_token() {
            println!("goto parse_number_expr");
            return self.parse_number_expr();
        }
        return self.log_error("Expected expression, got unknown token");
    }


    /// numberexpr
    ///     ::= number
    fn parse_number_expr(&mut self) -> Option<Box<ExprAST>> {
        let Token::Number {value} = self.lexer.last_token() else {
            return self.log_error("Expected number");
        };

        self.lexer.next(); // eat number
        
        Some(Box::new(ExprAST::NumberExprAST { val: value }))
    }

    /// parenexpr
    ///     ::= '(' expression ')'
    fn parse_paren_expr(&mut self) -> Option<Box<ExprAST>> {
        self.lexer.next(); // eat (.
        let inner = self.parse_expression();
        match self.lexer.last_token() {
            Token::Identifier{id} => {
                if id != ")" {
                    return self.log_error("Expected ')'");
                }
            },
            _ => return self.log_error("Expected ')'")
        }
        self.lexer.next(); // eat ')'
        return inner;
    }

    ///  expression
    ///     ::= primary binoprhs
    pub fn parse_expression(&mut self) -> Option<Box<ExprAST>> {
        let lhs = self.parse_primary();
        match lhs {
            None => None,
            Some(_) => self.parse_binop_rhs(0, lhs)
        }
    }


    /// identifierexpr
    ///   ::= identifier
    ///   ::= identifier '(' expression* ')'    <---- function definition/call or prototype;
    pub fn parse_identifier_expr(&mut self) -> Option<Box<ExprAST>> {
        println!("parse_identifier_expr");
        let Token::Identifier { id } = self.lexer.last_token() else {
           return self.log_error("parse_identifier_expr() expected identifier");
        };

        self.lexer.next(); // eat id

        if let Token::Identifier {id: id_next} = self.lexer.last_token() {
            // simple var
            if id_next != "(" {
                return Some(Box::new(ExprAST::VariableExprAST{name: id.to_string()})); 
            }
        } else {
            return Some(Box::new(ExprAST::VariableExprAST{name: id.to_string()})); 
        }

        // function call
        self.lexer.next(); // eat '('

        // get args
        let mut args = Vec::<Box<ExprAST>>::new();

        if !is_identifier(&self.lexer.last_token()) || get_id(&self.lexer.last_token()) != ")" {
            loop {
                println!("looping");
                let Some(arg) = self.parse_expression() else {
                    return None;
                };

                args.push(arg);

                if !is_identifier(&self.lexer.last_token()) {
                    return self.log_error("Expected identifier");
                }

                if let Token::Identifier {id} = self.lexer.last_token() {
                    if id == ")" {
                        continue;
                    } else if id == "," {
                        self.lexer.next(); 
                    } else {
                        return self.log_error("Expected ')' or ',' in arg list");
                    }
                } else {
                    return self.log_error("Expected ')' or ',' in arg list");
                };

            }
        }


        self.lexer.next(); // eat ')'
        return Some(Box::new(ExprAST::CallExprAST {
            callee: id.to_string(),
            args: args,
        }));
    }


    /// binoprhs
    ///     ::= ('+' primary)*
    pub fn parse_binop_rhs(&mut self, prec: i32, option_lhs: Option<Box<ExprAST>>) -> Option<Box<ExprAST>> { 
        println!("parse_binop_rhs");
        let Some(mut lhs) = option_lhs else {
            return self.log_error("lhs should be non-null");
        };

        loop {
            println!("looping in binop_rhs");
            let tok_prec = self.get_precedence(&self.lexer.last_token());
            if tok_prec < prec {
                // not a binop
                return Some(lhs);
            }

            // binop
            let Token::Identifier{id: binop_id} = self.lexer.last_token() else {
                return self.log_error("Expected binary operation identifier");
            };

            println!("Checking binop '{}", binop_id);

            if !self.prec.contains_key(&binop_id) {
                return self.log_error("Expected binary operator");
            }

            self.lexer.next(); // eat binop
            let Some(mut rhs) = self.parse_primary() else {
                return None;
            };

            let next_prec = self.get_precedence(&self.lexer.last_token());
            if tok_prec < next_prec {
                // case like: A + B * C
                let Some(new_rhs) = self.parse_binop_rhs(tok_prec + 1, Some(rhs)) else {
                    return None;
                };
                rhs = new_rhs;
            }

            lhs = Box::new(ExprAST::BinaryExprAST{
                op: binop_id.to_string(),
                lhs: lhs,
                rhs: rhs
            });
        }
    }


    fn parse_prototype(&mut self) -> Option<Box<ExprAST>> {
        let Token::Identifier { id: func_name } = self.lexer.last_token() else {
            return self.log_error("Expected identifier in prototype");
        };

        self.lexer.next(); // eat name

        let Token::Identifier { id } = self.lexer.last_token() else {
            return self.log_error("Expected '(' after identifier in prototype");
        };

        if id != "(" {
            return self.log_error("Expected '(' after identifier in prototype");
        }

        self.lexer.next(); // eat '('

        let mut func_args: Vec<String> = Vec::new();
        loop {
            let Token::Identifier { id: arg } = self.lexer.last_token() else {
                return self.log_error("Expected identifier in prototype arguments");
            };

            if arg == String::from(")") {
                break;
            }
            
            func_args.push(arg);

            self.lexer.next(); // eat identifier
        }

        return Some(Box::new(ExprAST::PrototypeAST { name: func_name, args: func_args }));
    }

    fn get_precedence(&self, tok: &Token) -> i32 {
        if let Token::Identifier { id } = tok {
            if self.prec.contains_key(id) {
                return self.prec[id];
            }
        }
        return -1;
    }

    fn log_error(&self, s: &str) -> Option<Box<ExprAST>> {
        eprintln!("[***] Error: {s}");
        None
    }

    fn log_error_p(&self, s: &str) -> Option<Box<ExprAST>> {
        eprintln!("[***] Error: {s}"); 
        None
    }
}

