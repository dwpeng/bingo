use super::config;
use clap::{Arg, Command};
use colored::*;

fn msg(msg: &str) -> String {
    format!("{} {}", msg, config::get_bingo_bin_dir().green())
}

fn build_parser() -> Command {
    Command::new("bingo")
        .about("Bingo")
        .subcommand(
            Command::new("cp")
                .about(msg("Copy executable to"))
                .arg(Arg::new("path").required(true).help("Path to executable"))
                .arg(Arg::new("name").help("Name of executable")),
        )
        .subcommand(
            Command::new("ln")
                .about(msg("Link executable to"))
                .arg(Arg::new("path").required(true).help("Path to executable"))
                .arg(Arg::new("name").help("Name of executable")),
        )
        .subcommand(
            Command::new("rm")
                .about(msg("Remove executable from "))
                .arg(Arg::new("name").required(true).help("Name of executable")),
        )
        .subcommand(
            Command::new("mv")
                .about(msg("Rename executable in "))
                .arg(Arg::new("old").required(true).help("Name of executable"))
                .arg(
                    Arg::new("new")
                        .required(true)
                        .help("New name of executable"),
                ),
        )
        .subcommand(Command::new("ls").about(msg("List executables in")))
        .subcommand(
            Command::new("run")
                .visible_alias("r")
                .about(msg("Run executable in "))
                .arg(Arg::new("name").required(true).help("Name of executable"))
                .arg(
                    Arg::new("args")
                        .action(clap::ArgAction::Append)
                        .default_missing_value("")
                        .help("Arguments"),
                ),
        )
}

fn run_executable(c: &config::BingoConfigFile, name: &str, args: Vec<String>, alert: bool) {
    let executable = &c.config.executables;
    let executable = executable.iter().find(|e| e.name == name);
    match executable {
        Some(e) => {
            let path = e.path.clone();
            let path = std::path::Path::new(&path);
            let status = std::process::Command::new(path).args(args).status();
            match status {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            if alert {
                eprintln!("Executable {} not found.", name);
                std::process::exit(1);
            }
        }
    }
}

static SUBCOMMANDS: &[&str] = &["cp", "ln", "rm", "mv", "ls", "run", "r"];

pub fn cli_run() {
    match config::BingoConfigFile::init() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
    let mut config_file = config::BingoConfigFile::new();
    match config_file.load() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
    {
        let args = std::env::args().collect::<Vec<String>>();
        if args.len() > 1 {
            let command = &args[1];
            let mut command_args = vec![];
            if !SUBCOMMANDS.contains(&command.as_str()) {
                // check if command is a executable
                if args.len() > 2 {
                    command_args = args[2..].to_vec();
                }
                let executables = &config_file.config.executables;
                let executable = executables.iter().find(|e| e.name == *command);
                if let Some(_) = executable {
                    run_executable(&config_file, command, command_args, false);
                    std::process::exit(0);
                }
            }
        }
    }

    let default_name = "".to_string();
    let matches = build_parser().get_matches();
    match matches.subcommand() {
        Some(("ln", args)) => {
            let path = args.get_one::<String>("path").unwrap().clone();
            let path = std::path::Path::new(&path);
            let mut name = args
                .get_one::<String>("name")
                .unwrap_or(&default_name)
                .clone();
            if name.len() == 0 {
                name = path.file_name().unwrap().to_str().unwrap().to_string();
                name = name.split('.').collect::<Vec<&str>>()[0].to_string();
            }
            match config_file.config.add_executable(
                &path,
                &name,
                config::ExecutableType::LinkBinary,
            ) {
                Ok(_) => {
                    println!("{} {} {}", "Linked".green(), name, "to".green());
                    println!("{}", path.display());
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
            config_file.save().unwrap();
        }
        Some(("cp", args)) => {
            let path = args.get_one::<String>("path").unwrap().clone();
            let path = std::path::Path::new(&path);
            let mut name = args
                .get_one::<String>("name")
                .unwrap_or(&default_name)
                .clone();
            if name.len() == 0 {
                name = path.file_name().unwrap().to_str().unwrap().to_string();
                name = name.split('.').collect::<Vec<&str>>()[0].to_string();
            }
            match config_file
                .config
                .add_executable(&path, &name, config::ExecutableType::Binary)
            {
                Ok(_) => {
                    println!("{} {} {}", "Copied".green(), name, "to".green());
                    println!("{}", path.display());
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
            config_file.save().unwrap();
        }
        Some(("rm", args)) => {
            let name = args.get_one::<String>("name").unwrap().clone();
            match config_file.config.remove_executable(&name) {
                Ok(_) => {
                    println!("{} {}", "Removed".green(), name);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
            config_file.save().unwrap();
        }
        Some(("mv", args)) => {
            let old = args.get_one::<String>("old").unwrap().clone();
            let new = args.get_one::<String>("new").unwrap().clone();
            match config_file.config.rename_executable(&old, &new) {
                Ok(_) => {
                    println!("{} {} {}", "Renamed".green(), old, "to".green());
                    println!("{}", new);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
            config_file.save().unwrap();
        }

        Some(("ls", _)) => {
            let executables = config_file.config.executables;
            if executables.len() == 0 {
                println!("No executables found.");
                std::process::exit(0);
            }
            for (mut index, e) in executables.iter().enumerate() {
                index += 1;
                match e.executable_type {
                    config::ExecutableType::Binary => {
                        println!("{index}: {} => {}", e.name, e.path.green());
                    }
                    config::ExecutableType::LinkBinary => {
                        println!("{index}: {} -> {}", e.name, e.path.cyan());
                    }
                }
            }
        }

        Some(("run", args)) => {
            let name = args.get_one::<String>("name").unwrap().clone();
            let args = args.get_many::<String>("args");

            let args = match args {
                Some(a) => a.map(|a| a.clone()).collect::<Vec<String>>(),
                None => vec![],
            };
            run_executable(&config_file, &name, args, true);
        }
        _ => {
            eprintln!("No command provided.");
            std::process::exit(1);
        }
    }
}
