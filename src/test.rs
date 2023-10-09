use crate::lexer::{Tokenizer, Token,};
use crate::parser::{Parser, ExprAST,};

use std::io::BufReader;

#[cfg(test)]
mod test_frontend {
    use std::env::args;

    use super::*;
    #[test]
    pub fn test_read() {
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
            let last_token = tokenizer.last_token();
            println!("last_token: {:?}", last_token);
            assert_eq!(exp, last_token);
            tokenizer.next(); 
        }
    }

    #[test]
    pub fn test_ast_single() {
        {
            let inp = "5";
            let expected_ast = Box::new(ExprAST::NumberExprAST { val: 5 as f64 });
            println!("comparing ast");
            test_input_ast(inp, Some(expected_ast));
            println!("ast compared");
        }

        {
            let inp = "var";
            let expected_ast = Box::new(ExprAST::VariableExprAST { name: String::from("var") });
            test_input_ast(inp, Some(expected_ast));
        }
    }

    #[test]
    pub fn test_ast_simple() {
        let num = "5 + 6";
        let expected_ast = Box::new(
            ExprAST::BinaryExprAST { 
                op: String::from("+"),
                lhs: Box::new(ExprAST::NumberExprAST { val: 5 as f64 }),
                rhs: Box::new(ExprAST::NumberExprAST { val: 6 as f64 }),
            }
        );
        test_input_ast(num, Some(expected_ast));
    }


    #[test]
    pub fn test_ast_long_arithmetic() {
        let long_arithmetic = "5 + (6 + 7) * 8";
        let expected_ast = Box::new(
            ExprAST::BinaryExprAST {
                op: String::from("+"),
                lhs: Box::new(ExprAST::NumberExprAST { val: 5_f64 }),
                rhs: Box::new(ExprAST::BinaryExprAST { 
                    op: String::from("*"), 
                    lhs: Box::new(ExprAST::BinaryExprAST { 
                        op: String::from("+"), 
                        lhs: Box::new(ExprAST::NumberExprAST { val: 6_f64 }), 
                        rhs: Box::new(ExprAST::NumberExprAST { val: 7_f64 }) 
                    }),
                    rhs: Box::new(ExprAST::NumberExprAST { val: 8_f64 })
                })
            }
        );
        test_input_ast(long_arithmetic, Some(expected_ast));
    }

    #[test]
    pub fn test_ast_with_variables() {
        let long_arithmetic = "5 + (var + 7) * k";
        let expected_ast = Box::new(
            ExprAST::BinaryExprAST {
                op: String::from("+"),
                lhs: Box::new(ExprAST::NumberExprAST { val: 5_f64 }),
                rhs: Box::new(ExprAST::BinaryExprAST { 
                    op: String::from("*"), 
                    lhs: Box::new(ExprAST::BinaryExprAST { 
                        op: String::from("+"), 
                        lhs: Box::new(ExprAST::VariableExprAST { name: String::from("var") }), 
                        rhs: Box::new(ExprAST::NumberExprAST { val: 7_f64 }) 
                    }),
                    rhs: Box::new(ExprAST::VariableExprAST { name: String::from("k") })
                })
            }
        );
        test_input_ast(long_arithmetic, Some(expected_ast));
    }

    fn test_input_ast<'a>(input: &str, expected_ast: Option<Box<ExprAST>>) {
        let mut bufreader = BufReader::new(input.as_bytes());
        println!("creating tokenizer");
        let mut lexer = Tokenizer::new(&mut bufreader);
        println!("creating parser");
        let mut parser = Parser::new(&mut lexer);
        println!("calling parser");
        let ast_result_opt = parser.parse_expression();
        assert!(compare_ast_opts(ast_result_opt, expected_ast));
    }

    fn compare_ast_opts(ast_result_opt: Option<Box<ExprAST>>, ast_expected_opt: Option<Box<ExprAST>>) -> bool {
        if let Some(ast_expected) = ast_expected_opt {
            let Some(ast_result) = ast_result_opt else {
                return false;
            };

            if let ExprAST::NumberExprAST { val: val_expected } = *ast_expected {
                if let ExprAST::NumberExprAST { val: val_result } = *ast_result {
                    return (val_expected - val_result).abs() < 1e-7;
                } else {
                    return false;
                }
            } else if let ExprAST::VariableExprAST { name: name_expected } = *ast_expected {
                if let ExprAST::VariableExprAST { name: name_result } = *ast_result {
                    return name_expected == name_result;
                } else {
                    return false;
                }
            } else if let ExprAST::BinaryExprAST { op: op_expected, lhs: lhs_expected, rhs: rhs_expected} = *ast_expected {
                if let ExprAST::BinaryExprAST { op: op_result, lhs: lhs_result, rhs: rhs_result } = *ast_result {
                    return op_expected == op_result
                            && compare_ast_opts(Some(lhs_expected), Some(lhs_result))
                            && compare_ast_opts(Some(rhs_expected), Some(rhs_result));
                } else {
                    return false;
                }
            } else if let ExprAST::PrototypeAST { name: name_expected, args: args_expected } = *ast_expected {
                if let ExprAST::PrototypeAST { name: name_result, args: args_result } = *ast_result {
                    return name_expected == name_result && args_expected == args_result;
                } else {
                    return false;
                }
            } else if let ExprAST::FunctionAST { proto: proto_expected, body: body_expected } = *ast_expected {
                if let ExprAST::FunctionAST { proto: proto_result, body: body_result } = *ast_result {
                    return compare_ast_opts(Some(proto_result), Some(proto_expected)) && compare_ast_opts(Some(body_result), Some(body_expected));
                } else {
                    return false;
                }
            } else if let ExprAST::CallExprAST { callee: callee_expected, args: args_expected} = *ast_expected {
                if let ExprAST::CallExprAST { callee: callee_result, args: args_result } = *ast_result {
                    if callee_expected != callee_result || args_expected.len() != args_result.len() {
                        return false;
                    }

                    for (arg_expected, arg_result) in args_expected.into_iter().zip(args_result.into_iter()) {
                        if !compare_ast_opts(Some(arg_expected), Some(arg_result)) {
                            return false;
                        }
                    }

                    return true;
                } else {
                    return false;
                }
            } else {
                unreachable!();
            }
        } else {
            return ast_expected_opt.is_none();
        }
    }

    pub fn id_tok(s: &str) -> Token {
        Token::Identifier { id: String::from(s) }
    }

    pub fn num_tok(v: f64) -> Token {
        Token::Number { value: v }
    }
}
