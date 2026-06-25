# Use the starter to run your projects!
I ran into a problem a while ago. To start all my api, I need 4 terminals (and two more for typescript and node.js). So, I wrote my code starter for rust projects.

Now it start docker, run bin projects and build libs in workspace directory (that specified on root .env file)

## First of all create workspace
Create Cargo.toml file with:
```
[workspace]
resolver = "3"
```
Now, you can create new bin or lib rust projects under workspace. Cargo automaticly add new project into Cargo.toml:
```
[workspace]
resolver = "3"
members = [
    "api/authorization",
    "api/chat", 
    "api/database", 
    "api/site", 
    "starter"
    ]
```
### Tips
- Don't forget use cargo autoinherit (```cargo install cargo-autoinherit```) to add common dependencies into workspace Cargo.toml
- In workspace every project have common .env file on root directory (probably it's not work for docker .env file, put into starter or another project, that run docker)

## Add projects names into .env 
Add path to docker-compose.yml, bin and lib projects name into root .env:
```
PATH_TO_DOCKER_YAML='./starter/database/docker'
MEMBERS_BIN='authorization chat site'
MEMBERS_LIB='database'
```

## To start project with starter use
```
cargo run -p starter
```

## Optional flags
#### ```--watch ```
- Used to start all project with cargo watch (depend on cargo-watch)
#### ```--full-restart```
- Make full restart of docker

### Expample
```
Cargo run -p starter -- --watch --full-restart
```