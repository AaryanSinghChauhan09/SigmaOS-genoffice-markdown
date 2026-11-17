use std::io::{self, Read};

fn usage() -> ! {
    eprintln!("usage: sigmaos_genoffice_markdown [--tokens] [FILE]");
    eprintln!("  renders Markdown to HTML on stdout; reads stdin when FILE is omitted");
    std::process::exit(2);
}

fn main() {
    let mut tokens = false;
    let mut path: Option<String> = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-t" | "--tokens" => tokens = true,
            "-h" | "--help" => usage(),
            _ if path.is_none() => path = Some(arg),
            _ => usage(),
        }
    }

    let input = match path {
        Some(p) => std::fs::read_to_string(&p).unwrap_or_else(|e| {
            eprintln!("error: cannot read {p}: {e}");
            std::process::exit(1);
        }),
        None => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s).expect("failed to read stdin");
            s
        }
    };

    if tokens {
        for token in sigmaos_genoffice_markdown::parse_markdown(&input) {
            println!("{token:?}");
        }
    } else {
        print!("{}", sigmaos_genoffice_markdown::markdown_to_html(&input));
    }
}
