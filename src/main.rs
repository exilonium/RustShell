#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use std::os::unix::process::CommandExt;

const BUILT_IN_COMMANDS: [&str;5] = ["echo","exit","type","pwd","cd"];

fn tokenize(input:&str) -> Vec<String>{
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    
    for c in input.chars(){
        match c{
            '\'' if !in_double => in_single = !in_single, // toggle the single quote only if not inside double
            '"' if !in_single => in_double = !in_double, // toggle the double quote only if not inside single
            // below we are building string of the current stuff inside the '' to not tokenize them
            ' ' |'\t' if !in_single && !in_double =>{ 
                if !current.is_empty(){
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),

        }
    }
    if !current.is_empty(){
        tokens.push(current); // pushing the last token if any
    }

    tokens
}

enum CommandLocation {
    Builtin,
    Executable(PathBuf),
    NotFound,
}

impl CommandLocation{
    fn resolve(command_name:&str) -> Self{
        if BUILT_IN_COMMANDS.contains(&command_name){
            return Self::Builtin;
        }
        if let Some(path_var) = std::env::var_os("PATH"){
            for dir in std::env::split_paths(&path_var){
                let full_path = dir.join(command_name);
                if !full_path.exists() {continue};

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(&full_path) {
                        if metadata.permissions().mode() & 0o111 == 0 {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                return Self::Executable(full_path);
            }
        }
        Self::NotFound
    }
    fn describe(&self,name:&str) -> String{
        match self {
            CommandLocation::Builtin => format!("{} is a shell builtin", name),
            CommandLocation::Executable(path) => format!("{} is {}", name, path.display()),
            CommandLocation::NotFound => format!("{}: not found", name),
        }
    }
}

enum Command{
    ExitCommand,
    EchoCommand {display_string:String},
    TypeCommand {command_name: String},
    ExternalCommand { program: String, args: Vec<String> },
    CommandNotFound,
    PwdCommand,
    CdCommand {target : String},
}

impl Command{
    fn from_input(input:&str) -> Self {
        let input=input.trim();

        // splitting tokens so that i can have the command and arguments
        let tokens: Vec<String> = tokenize(&input);

        if tokens.is_empty() {
            return Command::CommandNotFound;
        }
        let program = tokens[0].clone();
        let args = &tokens[1..];
        let cmd = match program.as_str() {
            "exit" => Command::ExitCommand,
            "echo" => Command::EchoCommand {
                display_string: args.join(" ")
            },
            "type" => {
                if args.is_empty() {
                    Command::CommandNotFound
                } else {
                    Command::TypeCommand {
                        command_name: args[0].clone()
                    }
                }
            },
            "pwd" => Command::PwdCommand,
            "cd" => {
                let target = if args.is_empty() || args[0]== "~"{
                    match std::env::var("HOME"){
                        Ok(home) => home,
                        Err(_) =>"/".to_string(),
                    } 
                }else {
                    args[0].clone()
                };
                return Command::CdCommand {target};
            },
            _ => {
                let loc = CommandLocation::resolve(&program);
                match loc {
                    CommandLocation::Executable(_) => {
                        Command::ExternalCommand { program,args: args.to_vec() }
                    }
                    CommandLocation::NotFound => Command::CommandNotFound,
                    CommandLocation::Builtin => unreachable!(), // handled above
                }
            }
        };

        cmd
    }


}

fn main() {
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        //implementing internal built_in_commands
        let command = Command::from_input(&input);
        match command{
            Command::ExitCommand => break,
            Command::EchoCommand {display_string} => println!("{}", display_string),
            Command::TypeCommand {command_name} => {
                let location = CommandLocation::resolve(&command_name);
                println!("{}", location.describe(&command_name));
            },
            Command::ExternalCommand {program,args} => {
                match CommandLocation::resolve(&program) {
                    CommandLocation::Executable(path) =>{
                        let child =std::process::Command::new(&path)
                            .arg0(&program)
                            .args(&args)
                            .spawn();
                        match child {
                            Ok(mut c) =>{
                                let _ = c.wait();
                            }
                            Err(_) =>{
                                println!("{}: failed to execute", program);
                            }
                        }
                    }
                    _ => println!("{}: command not found",program),

                }
            },
            Command::CommandNotFound =>{
               println!("{}: command not found", input.trim());
            },
            Command::PwdCommand => {
                match std::env::current_dir(){
                    Ok(path) => println!("{}",path.display()),
                    Err(_) => println!("pwd: error retrieving current directory"),
                }
            },
            Command::CdCommand {target} =>{
                if let Err(_) = std::env::set_current_dir(&target){
                    println!("cd: {}: No such file or directory",target);
                }
            },
        }
    }
}
