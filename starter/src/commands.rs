use std::collections::BTreeMap;


static mut PROCESS_MAP: BTreeMap<String, Process> = BTreeMap::new();

#[derive(Clone, Copy, Debug)]
pub enum Process {
    Unknown,
    Docker,
    Api(u32),
    Lib(u32)
}

pub struct RestartCommand {
    pub process: Process,
    pub full: bool
}

pub enum Command {
    Exit,
    Kill(Process),
    Restart(RestartCommand),
    Clear,
    Unknown
}

pub fn init() {
    let map_ref = unsafe {&mut *&raw mut PROCESS_MAP};
    
    let libs: Vec<String> = std::env::var("MEMBERS_LIB").unwrap_or_default()
        .split(" ").map(Into::into).collect();

    let apis: Vec<String> = std::env::var("MEMBERS_BIN").unwrap_or_default()
        .split(" ").map(Into::into).collect();
    
    // Insert libs
    for i in 0..libs.len() {
        map_ref.insert(libs[i].clone(), Process::Lib(i as u32));
    }

    // Insert apis
    for i in 0..apis.len() {
        map_ref.insert(apis[i].clone(), Process::Api(i as u32));
    }

    // Insert docker
    map_ref.insert("docker".into(), Process::Docker);
}

impl Command {
    pub fn parse(text: &str) -> Command {
        // load libs and apis
        let command = text.split(' ').next().unwrap();
        match command {
            "/exit" => {
                Command::Exit
            }
            "/kill" => {
                let map = unsafe {&mut *&raw mut PROCESS_MAP};

                let args: Vec<&str> = text.split(" ").collect();
                
                if args.len() < 2 {
                    return Command::Kill(Process::Unknown);
                }

                if let Some(&process) = map.get(args[1]) {
                    return Command::Kill(process);
                }
                else {
                    return Command::Kill(Process::Unknown);
                }

            }
            "/restart" => {
                let map = unsafe {&mut *&raw mut PROCESS_MAP};

                let args: Vec<&str> = text.split(" ").collect();

                if args.len() < 2 {
                    return Command::Restart(RestartCommand { process: Process::Unknown, full: false });
                }

                let mut full_res = false;
                let mut process = Process::Unknown;

                for &arg in &args {
                    if let Some(&process_temp) = map.get(arg) {
                        process = process_temp;
                    } 
                    else if arg == "--full" {
                        full_res = true;
                    }
                }
                return Command::Restart(RestartCommand { process: process, full: full_res });
            }
            "/clear" => {
                return Command::Clear;
            }

            _ => {
                Command::Unknown
            }
        }
    }
}