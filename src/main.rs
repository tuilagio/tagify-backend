
use actix_web::{
    HttpServer,
    App,
    middleware::Logger
};
use actix_files as fs;

use listenfd::ListenFd;
use tokio_postgres::{NoTls};
use std::fs::File;
use std::io::Read;


 
//new
mod config;
mod errors;
mod models;
mod handlers;

use crate::handlers::*;



#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    // Read config
    let conf = crate::config::MyConfig::new("Settings").unwrap();

    // Create db connection pool
    let pool = conf.postgres.create_pool(NoTls).unwrap();

    // Create connection to database
    let client = pool.get().await.expect("Could not connect to database");

    // Read schema.sql and create db table
    let mut schema = String::new();
    File::open("schema.sql")?.read_to_string(&mut schema)?;
    client.batch_execute(&schema).await.expect("Failed while creating a new database");

    // Build server address
    let ip = conf.server.hostname + ":" + &conf.server.port;
    println!("Server is reachable at http://{}", ip);

    // Setup logging
    std::env::set_var("RUST_LOG", "DEBUG");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // Register http routes
    let mut server = HttpServer::new(move|| {
        App::new()
            // Enable logger
            .wrap(Logger::default())
            // Give every handler access to the db connection pool
            .data(pool.clone())
            // Serve every file in directory from ../dist
            .service(fs::Files::new("/app/debug_dist", "../debug_dist").show_files_listing())
            // Register handlers
            .service(index)
            .service(get_all_todos)
            .service(add_todo)
            .service(delete_todo)
            .service(update_todo)
    });

    // Enables us to hot reload the server
    let mut listenfd = ListenFd::from_env();
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind(ip)?
    };

    server.run().await
}