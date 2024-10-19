use std::{env, process};

pub fn get_editor_binary_name() -> String {
    match env::var("EDITOR") {
        Ok(editor) => editor,
        Err(_) => {
            eprintln!("Error: The EDITOR environment variable is not set.");
            eprintln!("You can set it e.g. by running: export EDITOR=vim");
            eprintln!("Or for a single run: EDITOR=vim {}", env::args().next().unwrap_or_else(|| String::from("program_name")));
            process::exit(1);
        }
    }
}

pub fn get_terminal_binary_name() -> String {
    match env::var("TERMINAL") {
        Ok(terminal) => terminal,
        Err(_) => {
            eprintln!("Error: The TERMINAL environment variable is not set.");
            eprintln!("You can set it e.g. by running: export TERMINAL=xterm");
            eprintln!("Or for a single run: TERMINAL=xterm {}", env::args().next().unwrap_or_else(|| String::from("program_name")));
            process::exit(1);
        }
    }
}