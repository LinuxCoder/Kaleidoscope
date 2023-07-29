use crate::lexer::Tokenizer;

use crate::lexer::Token;

use std::io::BufReader;

#[cfg(test)]
mod TestTokenizer {
    use super::*;
    #[test]
    pub fn testRead() {
        let mut bufreader = BufReader::new(
            r#"
                def fib(x)
                    if x < 3 then
                        1
                    else
                        fib(x - 1) + fib(x - 2)
            "#.as_bytes());
        let mut tokenizer = Tokenizer::new(&mut bufreader);
        let expected = [Token::Def, id_tok("fib"), id_tok("("), id_tok("x"), id_tok(")"),
                id_tok("if"), id_tok("x"), id_tok("<"), num_tok(3.0), id_tok("then"),
                num_tok(1.0),
                id_tok("else"),
                id_tok("fib"), id_tok("("), id_tok("x"), id_tok("-"), num_tok(1.0), id_tok(")"),
                    id_tok("+"), id_tok("fib"), id_tok("("), id_tok("x"), id_tok("-"), num_tok(2.0), id_tok(")"),
                Token::Eof
                ];

        for exp in expected {
            let last_token: &Token = tokenizer.LastToken();
            println!("last_token: {:?}", *last_token);
            assert_eq!(exp, *last_token);
            tokenizer.Next(); 
        }
    }

    pub fn id_tok(s: &str) -> Token {
        Token::Identifier { id: String::from(s) }
    }

    pub fn num_tok(v: f64) -> Token {
        Token::Number { value: v }
    }
}
