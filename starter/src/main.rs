mod docker;
mod commands;


use std::process::{Child, ChildStdout, Command, Stdio};
use std::io::{self, BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::env;

use colored::Colorize;
use commands::Process;
use dotenv;


// Projects files struct
#[derive(Debug, Clone)]
struct ProjectFile {
    name: String,
    is_worked: Arc<AtomicBool>
}

// Print info functions 
fn ok_message(text: &str) {
    println!("{} {}", format!("[INFO]").green(), text);
}

fn info_message(text: &str) {
    println!("{} {}", format!("[INFO]").yellow(), text);
}

fn err_message(text: &str) {
    println!("{} {}", format!("[INFO]").red(), text);
}

// Create command
fn create_command(name: &String, cargo_watch: &bool, build: bool) -> (Child, ChildStdout){
        let run = if build {"build"} else {"run"};
        let mut command = Command::new("cargo");

        // cargo watch
        if *cargo_watch {
            command.args(["watch", "-x", &format!("{} -p {}", run, name)]);
        }
        else {
            command.args([run, "-p", name]);
        }

        // Create child and get stdout
        let mut child = command
            .stdout(Stdio::piped())
            .spawn()
            .expect(&format!("failed to execute process").red());

        let stdout = child.stdout.take().expect("no stdout");

        (child, stdout)
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    commands::init();

    // Default values
    let default_path_docker_yaml: String = "./starter/database/docker".into();
    let default_members_bin: String = "".into();
    let default_members_lib: String = "".into();

    // .env
    let path_to_docker_yaml = env::var("PATH_TO_DOCKER_YAML").unwrap_or_else(|_| {
        info_message("Path to docker not set!");
        default_path_docker_yaml
    });

    let mut apis: Vec<ProjectFile>  = env::var("MEMBERS_BIN").unwrap_or_else(|_| {
        info_message("Apis list not set!");
        default_members_bin
    }).split(" ").map(|api| {ProjectFile { name: api.into(), is_worked: Arc::new(false.into()) }}).collect();
    let mut libs: Vec<ProjectFile> = env::var("MEMBERS_LIB").unwrap_or_else(|_| {
        info_message("Apis list not set!");
        default_members_lib
    }).split(" ").map(|lib| {ProjectFile { name: lib.into(), is_worked: Arc::new(false.into()) }}).collect();

    let mut apis_command: Vec<Child> = vec![];
    let mut libs_command: Vec<Child> = vec![]; 

    // Args
    let args: Vec<String> = env::args().collect();
    // Cargo watch argument
    let mut cargo_watch = false;
    // Docker full restart argument
    let mut full_restart = false;

    // Get arguments
    if args.len() > 1 {
        for argument in args {
            if argument == "--watch" {
                cargo_watch = true;
                break;
            }
            if argument == "--full-restart" {
                full_restart = true;
            }
        }        
    }
    

    /*            Database            */
    let docker = docker::Docker::new(path_to_docker_yaml);
    let _ = docker.start(full_restart);


    /*              BIN and LIB               */
    // Start project bin
    for api in &mut apis{
        let (child, stdout) = create_command(&api.name, &cargo_watch, false);

        let async_api_name = api.name.clone();
        let async_is_worked = api.is_worked.clone();

        // Reading stdout
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while let Ok(cmd_line) = reader.read_line(&mut line) {
                if cmd_line == 0 {break;}
                println!("{} {}", format!("[{} api]", async_api_name).green(), line);
            }
            async_is_worked.store(false, Ordering::Relaxed);
            info_message(&format!("{} stdout killed!", async_api_name));
        });

        api.is_worked.store(true, Ordering::Relaxed);

        // add to vector
        apis_command.push(child);
    }

    // Build project libs
    for lib in &mut libs{
        let (child, _) = create_command(&lib.name, &cargo_watch, true);

        lib.is_worked.store(true, Ordering::Relaxed);
        libs_command.push(child);
    }

    // Get commands
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let text: &str = line.as_ref().unwrap();
        match commands::Command::parse(text) {
            /*                              EXIT                                 */
            commands::Command::Exit => {
                for i in 0..apis_command.len() {
                    let _ = apis_command[i].kill();
                }
                for i in 0..libs_command.len() {
                    let _ = libs_command[i].kill();
                }

                let _ = docker.stop();

                ok_message("All apis, libs and docker were killed! Leaving main function!");
                return;
            }
            /*                              KILL                                 */
            commands::Command::Kill(process) => {
                match process {
                    Process::Docker => {
                        if docker.stop().is_ok() {
                            ok_message("Docker stopped!");
                        }
                        else {
                            err_message("Docker cannot be stopped!");
                        }
                    }
                    Process::Lib(lib_id) => {
                        if libs[lib_id as usize].is_worked.load(Ordering::Relaxed) == false {
                            err_message("Lib already killed!");
                        }
                        else {
                            let _ = libs_command[lib_id as usize].kill();
                            libs[lib_id as usize].is_worked.store(false, Ordering::Relaxed);
                            ok_message(&format!("Lib {} killed!", libs[lib_id as usize].name));
                        }

                    }
                    Process::Api(api_id) => {
                        if apis[api_id as usize].is_worked.load(Ordering::Relaxed) == false {
                            err_message("Api already killed!");
                        }
                        else {
                            let _ = apis_command[api_id as usize].kill();
                            apis[api_id as usize].is_worked.store(false, Ordering::Relaxed);
                            ok_message(&format!("Api {} killed!", apis[api_id as usize].name));
                        }
                    }
                    _ => {
                        info_message("Unknown process name");
                    }
                }
            }
            /*                              REST                                 */
            commands::Command::Restart(rest_command) => {
                match rest_command.process{
                    Process::Docker => {
                        let _ = docker.stop();
                        if docker.start(rest_command.full).is_ok() {
                            ok_message("Docker started!");
                        }
                        else {
                            err_message("Docker cannot be started!");
                        }   
                    }
                    Process::Api(api_id) => {
                        let _ = apis_command[api_id as usize].kill();
                        let (child, stdout) 
                            = create_command(&apis[api_id as usize].name, &cargo_watch, false);

                        let async_api_name = apis[api_id as usize].name.clone();
                        // Reading stdout
                        tokio::spawn(async move {
                            let mut reader = BufReader::new(stdout);
                            let mut line = String::new();
                            while let Ok(cmd_line) = reader.read_line(&mut line) {
                                if cmd_line == 0 {break;}
                                println!("{} {}", format!("[{} api]", async_api_name).green(), line);
                            }
                            info_message(&format!("{} stdout killed!", async_api_name));
                        });
                        
                        apis_command[api_id as usize] = child;
                        apis[api_id as usize].is_worked.store(true, Ordering::Relaxed);
                        ok_message(&format!("Api {} restarted!", apis[api_id as usize].name)); 
                    }
                    Process::Lib(lib_id) => {
                        let _ = libs_command[lib_id as usize].kill();
                        (libs_command[lib_id as usize], _) 
                            = create_command(&libs[lib_id as usize].name, &cargo_watch, true);
                        libs[lib_id as usize].is_worked.store(true, Ordering::Relaxed);
                        ok_message(&format!("Lib {} restarted!", libs[lib_id as usize].name));  
                    }
                    _ => {info_message("Unknown process name");}
                }
            }
            commands::Command::Clear => {
                let _ = Command::new("clear").spawn().expect("Cannot spawn clear command");
            }
            _ => {
                println!("\"{}\" {}", text, format!("Command not found!").yellow());
            }
        }
    }
}
