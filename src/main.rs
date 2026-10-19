use clap::Parser as ClapParser;
use func::{
    interpreter::{Interpreter, Value},
    parser::Parser as AstParser,
    scanner::scan,
};
use rustyline::{
    Config, DefaultEditor,
    error::ReadlineError::{self},
};
use std::{error::Error, fs, path::PathBuf};

const LANG_NAME: &str = "Fun Programming Language";
const LANG_VERSION: &str = "0.0.1";

#[derive(ClapParser, Debug)]
#[command(
    name = LANG_NAME,
    version,
    about = LANG_NAME
)]
struct Args {
    file: Option<PathBuf>,

    #[arg(short, long, help = "Print the AST and env when evaluating")]
    debug: bool,
}

fn run_source(vm: &mut Interpreter, src: &str) -> Result<(), Box<dyn Error>> {
    let tokens = scan(&src).map_err(|e| format!("{e:#?}"))?;
    let mut parser = AstParser::new(&tokens);
    let tree = parser.parse().map_err(|e| format!("{e:#?}"))?;
    let result = vm.run(tree).map_err(|e| format!("{e:#?}"))?;

    if !matches!(result, Value::Nil) {
        println!("{}", result);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut vm = Interpreter::new(args.debug);

    // Execute program by source file
    if let Some(path) = args.file {
        let src = fs::read_to_string(&path)?;
        return run_source(&mut vm, &src);
    }

    let repl_config = Config::builder()
        .edit_mode(rustyline::EditMode::Vi)
        .auto_add_history(false)
        .build();

    let mut repl = DefaultEditor::with_config(repl_config)?;

    // Enter Repl mode
    println!("{} v{}", LANG_NAME, LANG_VERSION);
    loop {
        match repl.readline(">>> ") {
            Ok(line) => {
                let src = line.trim();
                if src == "quit" || src == "exit" {
                    break;
                } else if src.is_empty() {
                    continue;
                }
                let _ = repl.add_history_entry(src);

                // TODO add better app error handling here
                if let Err(e) = run_source(&mut vm, src) {
                    eprintln!("{e}");
                }
            }
            Err(ReadlineError::Interrupted) => {
                // CTRL + C
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // CTRL + D
                println!("");
                break;
            }
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        }
    }

    Ok(())
}
