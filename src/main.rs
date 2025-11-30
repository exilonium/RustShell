#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;

const BUILT_IN_COMMANDS: [&str;3] = ["echo","exit","type"];

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
    CommandNotFound,
}

impl Command{
    fn from_input(input:&str) -> Self {
        let input=input.trim();
        if input == "exit" || input == "exit 0" {
            return Self::ExitCommand;
        };
        if let Some(pos) = input.find("echo") {
            if pos ==0{
                return Self::EchoCommand{
                    display_string: input["echo".len()..].trim().to_string(),
                };
            }
        }
        if let Some(pos) = input.find("type"){
            if pos==0 {
                return Self::TypeCommand{
                    command_name: input["type".len()..].trim().to_string(),
                };
            }
        }
        Self::CommandNotFound // we are returning this value
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
            Command::CommandNotFound => println!("{}: command not found", input.trim()),
        }
    }
}
