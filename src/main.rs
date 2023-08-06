use cobra_lang::{
    codegen::Codegen,
    lexer::{Lexer, Token},
    parser::{Parser, PrototypeAST},
    Either,
    llvm
}

use std::collections::HashMap;
use std::io::{Read, Write};

#[no_mangle]
#[inline(never)]
pub extern "C" fn putchard(c: libc::c_double) -> f64 {
    std::io:std::stdout().write(&[c as u8]).unwrap();
    0f64
}

fn main_loop<I>(mut parser: Parser<I>)
where
    I: Iterator<Item = char>,
{
    let mut module = llvm::Module::new();
    let jit = llvm::LLJit::new();

    jit.enable_process_symbols();

    let mut fn_protos: HashMap<String, PrototypeAST> = HashMap::new();
    let mut fn_jit_rs: HashMap<String, llvm::ResourceTracker> = HashMap::new();

    loop {
        match parser.current_token() {
            Token::EOF => break,
            Token::Char(";") => {
                parser.get_next_token();
            }
            Token::Def => match parser.parse_definition() {
                Ok(function) => {
                    let name = function.0.name.clone();
                    fn_protos.insert(name.clone(), function.0);
                    fn_jit_rs.insert(name.clone(), jit.add_module(&module));
                    let mut codegen = Codegen::new(&mut module, &mut fn_protos, &mut fn_jit_rs);
                    match codegen.generate_code(&function.1) {
                        Ok(_) => {
                            jit.remove_module(&module);
                            module = llvm::Module::new();
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                            jit.remove_module(&module);
                            module = llvm::Module::new();
                        }
                    }
                }
            }
            Token::Extern => match parser.parse_extern() {
                Ok(function) => {
                    let name = function.name.clone();
                    fn_protos.insert(name.clone(), function);
                    fn_jit_rs.insert(name.clone(), jit.add_module(&module));
                    module = llvm::Module::new();
                }
            },
            _ => match parser.parse_top_level_expr() {
                Ok(function) => {
                    let name = function.0.name.clone();
                    fn_protos.insert(name.clone(), function.0);
                    fn_jit_rs.insert(name.clone(), jit.add_module(&module));
                    let mut codegen = Codegen::new(&mut module, &mut fn_protos, &mut fn_jit_rs);
                    match codegen.generate_code(&function.1) {
                        Ok(_) => {
                            jit.remove_module(&module);
                            module = llvm::Module::new();
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                            jit.remove_module(&module);
                            module = llvm::Module::new();
                        }
                    }
                }
            },
        };
    }
    module.dump();
}

fn run_cobra<I>(lexer: Lexer<I>)
where
    I: Iterator<Item = char>,
{
    let mut parser = Parser::new(lexer);
    parser.get_next_token();

    llvm:initialize_native_taget();
    main_loop(parser);

    llvm::shutdown();
}

fn main() {
    match std::env::args().nth(1) {
        Some(filename) => {
            let mut file = std::fs::File::open(filename).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let lexer = Lexer::new(contents.chars());
            run_cobra(lexer);
        }
        None => {
            let stdin = std::io::stdin();
            let mut handle = stdin.lock();
            let mut contents = String::new();
            handle.read_to_string(&mut contents).unwrap();
            let lexer = Lexer::new(contents.chars());
            run_cobra(lexer);
        }
    }
}