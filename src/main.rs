use std::{env, fs, process};

fn main() {
    let mut args = env::args().skip(1);
    let Some(input) = args.next() else {
        eprintln!("usage: jordan-calculus <source.jc> [out.wat]\n       jordan-calculus --expr '<expression>' [out.wat]");
        process::exit(2);
    };

    let (source, out) = if input == "--expr" {
        let Some(expr) = args.next() else { eprintln!("missing expression after --expr"); process::exit(2); };
        (expr, args.next())
    } else {
        let source = fs::read_to_string(&input).unwrap_or_else(|e| { eprintln!("failed to read {input}: {e}"); process::exit(1); });
        (source, args.next())
    };

    let ast = jordan_calculus::parse(&source).unwrap_or_else(|e| { eprintln!("parse error: {e}"); process::exit(1); });
    let wat = jordan_calculus::compile_wat(&ast);
    if let Some(path) = out {
        fs::write(&path, wat).unwrap_or_else(|e| { eprintln!("failed to write {path}: {e}"); process::exit(1); });
    } else {
        println!("{wat}");
    }
}
