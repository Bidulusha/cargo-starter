use std::process::{Child, Command};
use colored::Colorize;
use std::cell::RefCell;

pub struct Docker{
    path_to_yml: String,
    child: RefCell<Option<Child>>
}

fn ok_message(text: &str) {
    println!("{} {}", format!("[DOCKER]").green(), text);
}

impl Docker{
    pub fn new(path_to_yml: String) -> Self {
        Docker {
            path_to_yml: path_to_yml,
            child: RefCell::new(None)
        }
    }

    pub fn start(&self, full_restart: bool) -> Result<(), ()> {
        if full_restart {
            Command::new("docker")
                .args(["compose", "down", "-v"])
                .current_dir(&self.path_to_yml)
                .spawn()
                .expect(&format!("{} {}", format!("[DOCKER]").red(), "INSTANCE CREATION ERROR"));
        }

        if self.is_start().is_ok() { 
            println!("{} {}", format!("[DOCKER]").yellow(), "Docker is already running!");
            return Err(()); 
        }

        *self.child.borrow_mut() = Some(
            Command::new("docker")
                .args(["compose", "up", "-d"])
                .current_dir(&self.path_to_yml)
                .spawn()
                .expect(&format!("{} {}", format!("[DOCKER]").red(), "INSTANCE CREATION ERROR"))
        );
        ok_message("docker is running now");
        Ok(())
    }
    
    pub fn stop(&self) -> Result<(), ()> {
        if self.is_start().is_err() { 
            println!("{} {}", format!("[DOCKER]").red(), "ERROR! DOCKER NOT STARTED!");
            return Err(()); 
        }
        if self.child.take().unwrap().kill().is_err() { return Err(()); };
        *self.child.borrow_mut() = None; 
        
        // if self.child.take().unwrap().kill().is_err() { return Err(()); };
        // let mut child = self.child.borrow_mut();
        // *child = None;

        Ok(())
    }

    fn is_start(&self) -> Result<(), ()> {
        if self.child.borrow().is_none() {
            return Err(());
        }        
        Ok(())
    }
}