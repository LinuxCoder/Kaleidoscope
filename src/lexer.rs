#[derive(Debug, Clone, PartialEq)]
pub enum Token { 
    Eof, 
    Def, 
    Extern,
    Identifier { id : String },
    Number { value : f64 },
}

pub fn is_identifier(tok: &Token) -> bool {
    match tok { 
        Token::Identifier {..} => true,
        _ => false
    }
}

pub fn get_id(tok: &Token) -> String {
    match tok {
        Token::Identifier{id} => id.to_string(),
        _ => unreachable!()
    }
}

pub struct Tokenizer<'a> {
    input: &'a mut dyn std::io::Read,
    last_char: char,
    last_token: Token, 
    eof_reached: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a mut impl std::io::Read) -> Self {
        let mut result = Self {
            input: input,
            last_char: ' ',
            last_token: Token::Eof,
            eof_reached: false
        };
        result.next();
        return result;
    }

    pub fn next(&mut self) {
        self.next_();
    }
    
    pub fn next_(&mut self) {
        if self.eof_reached {
            self.last_token = Token::Eof;
            return;
        }

        while self.last_char == ' ' || self.last_char == '\n' || self.last_char == '\t' {
            if !self.next_char() {
                if self.eof_reached {
                    self.last_token = Token::Eof;
                    return;
                } else {
                    println!("!!!! Unknown error");
                    assert_eq!(true, false);
                }
            }
        }
    
        if self.last_char.is_alphabetic() {
            let mut identifier: String = self.last_char.to_string();
            
            while self.next_char() && self.last_char.is_alphanumeric() {
                println!("last_char: {}", self.last_char);
                identifier.push(self.last_char as char);
                
            }
    
            if identifier == "def" {
                self.last_token = Token::Def;
                return;
            }
    
            if identifier == "extern" {
                self.last_token = Token::Extern;
                return;
            }

            self.last_token = Token::Identifier { id: identifier };
        } else if self.last_char.is_digit(10) {
            let mut num_str = String::new();
            num_str.push(self.last_char);
            while self.next_char() && (self.last_char.is_digit(10) || self.last_char == '.') {
                num_str.push(self.last_char);
            }
            let num: f64 = num_str.parse().unwrap();
            self.last_token = Token::Number {value: num};
        } else if self.last_char == '#' {
            while self.next_char() && self.last_char != '\n' {    
            }
            if !self.eof_reached {
                self.next_char(); // skip current '\n'
                self.next();
            }
        } else if self.eof_reached {
            // is EOF, return tok_eof
            self.last_token = Token::Eof;
        } else {
            // should be one of '+', '-', '*', '(', ')'
            let ch = String::from(self.last_char);
            self.next_char();
            self.last_token = Token::Identifier { id: ch };
        }
    }

    fn next_char(&mut self) -> bool {
        let res = self.next_char_();
        res
    }

    fn next_char_(&mut self) -> bool {
        let mut ch: &mut [u8] = &mut [0];
        match self.input.read(&mut ch) {
            Ok(0) => { self.eof_reached = true; return false },
            Ok(idx) => { println!("read: {}, char: {}", idx, ch[0]); self.last_char = ch[0] as char; return true },
            Err(_) => return false,
        }
    }

    pub fn last_token(&self) -> Token {
        self.last_token.clone()
    }
}

