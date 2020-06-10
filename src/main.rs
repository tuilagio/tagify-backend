#![cfg_attr(feature = "strict", deny(warnings))]

use actix_files as fs;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware::Logger, web, App, HttpServer};

use listenfd::ListenFd;
use std::fs::File;
use std::io::Read;
use tokio_postgres::NoTls;

mod config;
mod db;
mod errors;
mod handlers;

mod admin_handlers;

mod models;

use crate::admin_handlers::*;
use crate::handlers::*;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Read config
    let conf = crate::config::MyConfig::new("Settings").expect("Could not find Settings file");

    // Create db connection pool
    let pool = conf.postgres.create_pool(NoTls).unwrap();

    // Create connection to database
    let client = pool
        .get()
        .await
        .expect("Could not connect to postgres database");

    // Read schema.sql and create db table
    let mut schema = String::new();
    File::open("schema.sql")?.read_to_string(&mut schema)?;
    client
        .batch_execute(&schema)
        .await
        .expect("Failed while creating a new database");

    // Build server address
    let ip = conf.server.hostname + ":" + &conf.server.port;
    println!("Server is reachable at http://{}", ip);

    // Setup logging
    std::env::set_var("RUST_LOG", "DEBUG");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let cookie_key = conf.server.key;
    // Register http routes
    let mut server = HttpServer::new(move || {
        App::new()
            // Serve every file in directory from ../dist
            .service(fs::Files::new("/app/debug_dist", "../debug_dist").show_files_listing())
            // Give every handler access to the db connection pool
            .data(pool.clone())
            // Enable logger
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(cookie_key.as_bytes())
                    .name("auth-cookie")
                    .path("/")
                    .secure(false),
            ))
            //TODO maybe we need to change it :/
            //limit the maximum amount of data that server will accept
            .data(web::JsonConfig::default().limit(4096))
            //normal routes
            .service(web::resource("/").route(web::get().to(status)))
            // .configure(routes)
            .service(
                web::scope("/api")
                    //guest endpoints
                    .service(web::resource("/user_login").route(web::post().to(login)))
                    .service(web::resource("/user_logout").route(web::post().to(logout)))
                    //all admin endpoints
                    .service(
                        web::scope("/admin")
                            // .wrap(AdminAuthMiddleware)
                            .service(
                                web::resource("/create_admin").route(web::post().to(create_admin)),
                            )
                            .service(
                                web::resource("/delete_admin/{username}/{_:/?}")
                                    .route(web::delete().to(delete_admin)),
                            )
                            // interact with user
                            .service(
                                web::resource("/delete_account/{username}/{_:/?}")
                                    .route(web::delete().to(delete_account)),
                            ),
                    )
                    //user auth routes
                    .service(
                        web::scope("/auth")
                            // .wrap(AuthMiddleware)
                            .service(web::resource("/get_user").route(web::get().to(get_user))),
                    ),
            )
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
