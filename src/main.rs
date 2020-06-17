#![cfg_attr(feature = "strict", deny(warnings))]

use actix_files as fs;
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
mod my_identity_service;
mod my_cookie_policy;

mod models;

use crate::handlers::{logout, login, status};
use models::{ROLES};

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

    // Create default admin accounts
    db::create_user(&client, &conf.default_admin).await.expect("Could not create user account");

    // Create default user accounts
    db::create_user(&client, &conf.default_user).await.expect("Could not create default user account");

    let temp = conf.server.key.clone();

    // Register http routes
    let mut server = HttpServer::new(move || {
        let cookie_key = temp.as_bytes();

        let cookie_factory_user =  my_cookie_policy::MyCookieIdentityPolicy::new(cookie_key)
            .name(ROLES[1])
            .path("/")
            .secure(false);

        let cookie_factory_admin =  my_cookie_policy::MyCookieIdentityPolicy::new(cookie_key)
            .name(ROLES[0])
            .path("/")
            .secure(false);
        App::new()

            // Give login handler access to cookie factory
            .data(cookie_factory_user.clone())

            // Serve every file in directory from ../dist
            .service(fs::Files::new("/app/debug_dist", "../debug_dist").show_files_listing())
            // Give every handler access to the db connection pool
            .data(pool.clone())
            // Enable logger
            .wrap(Logger::default())

            //limit the maximum amount of data that server will accept
            .data(web::JsonConfig::default().limit(4096)) // max 4MB json
            //normal routes
            .service(web::resource("/").route(web::get().to(status)))
            // .configure(routes)
            .service(
                web::scope("/api")
                    //all admin endpoints
                    .service(web::resource("/login").route(web::post().to(login)))
                    .service(
                        web::scope("/admin")
                            .wrap(my_identity_service::IdentityService::new(
                                cookie_factory_admin,
                                pool.clone(),
                            ))
                            .service(web::resource("/logout").route(web::post().to(logout)))
                            .service(
                                web::resource("/user/{id}")
                                    .route(web::delete().to(admin_handlers::delete_user)),
                            )
                            .service(
                                web::resource("/user/{id}")
                                    .route(web::put().to(admin_handlers::update_user)),
                            )
                            .service(
                                web::resource("/user")
                                    .route(web::post().to(admin_handlers::create_user)),
                            )
                            // .service(
                            //     web::resource("/user/{id}")
                            //         .route(web::get().to(admin_handlers::get_user)),
                            // )
                    )
                    //user auth routes
                    .service(
                        //TODO: More then 3 routes make the last one not accessible
                        web::scope("/user")
                            .wrap(my_identity_service::IdentityService::new(
                                cookie_factory_user,
                                pool.clone(),
                            ))
                            .service(web::resource("/user").route(web::get().to(handlers::get_user)))
                            .service(web::resource("/logout").route(web::post().to(logout)))
                            .service(
                                web::resource("/user")
                                    .route(web::delete().to(handlers::delete_user)),
                            )
                            .service(
                                web::resource("/user")
                                    .route(web::put().to(handlers::update_user)),
                            )
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
