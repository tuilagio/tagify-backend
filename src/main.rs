#![cfg_attr(feature = "strict", deny(warnings))]

use actix_files as fs;
use actix_files::NamedFile;
use actix_web::{middleware::Logger, web, App, HttpServer, Result};
use std::path::PathBuf;

use listenfd::ListenFd;
use log::info;
use std::fs::File;
use std::io::Read;
use tokio_postgres::NoTls;

mod config;
mod db;
mod errors;
mod handlers;

mod admin_handlers;
mod my_cookie_policy;
mod my_identity_service;

mod models;
use crate::handlers::{login, logout, status};
use models::ROLES;

async fn index() -> Result<NamedFile> {
    let path: PathBuf = PathBuf::from("../debug_dist/index.html");
    Ok(NamedFile::open(path)?)
}

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
    match db::create_user(&client, &conf.default_admin).await {
        Ok(_item) => info!("Created default admin account"),
        Err(_e) => info!("Default user already exists"),
    }

    // Create default user accounts
    match db::create_user(&client, &conf.default_user).await {
        Ok(_item) => info!("Created default user"),
        Err(_e) => info!("Default user already exists"),
    }

    let temp = conf.server.key.clone();

    // Register http routes
    let mut server = HttpServer::new(move || {
        let cookie_key = temp.as_bytes();

        let cookie_factory_user = my_cookie_policy::MyCookieIdentityPolicy::new(cookie_key)
            .name(ROLES[1])
            .path("/")
            .secure(false);

        let cookie_factory_admin = my_cookie_policy::MyCookieIdentityPolicy::new(cookie_key)
            .name(ROLES[0])
            .path("/")
            .secure(false);
        App::new()
            // Give login handler access to cookie factory
            .data(cookie_factory_user.clone())
            // Serve every file in directory from ../dist
            .service(fs::Files::new("/app/debug_dist", "../debug_dist").show_files_listing())
            // Serve index.html
            .route("/", web::get().to(index))
            // Give every handler access to the db connection pool
            .data(pool.clone())
            // Enable logger
            .wrap(Logger::default())
            //limit the maximum amount of data that server will accept
            .data(web::JsonConfig::default().limit(4096)) // max 4MB json
            // .configure(routes)
            //status
            .service(
                web::scope("/api")
                    //all admin endpoints
                    .service(web::resource("/status").route(web::get().to(status)))
                    .service(web::resource("/login").route(web::post().to(login)))
                    .service(
                        web::scope("/admin")
                            .wrap(my_identity_service::IdentityService::new(
                                cookie_factory_admin,
                                pool.clone(),
                            ))
                            .route("/logout", web::delete().to(logout))
                            //get all users
                            .route("/users", web::get().to(status))
                            //create new user account
                            .route("/users", web::post().to(admin_handlers::create_user))
                            //get user by id
                            .route("/user/{user_id}", web::get().to(status))
                            //change user password
                            .route(
                                "/user/{user_id}",
                                web::put().to(admin_handlers::update_user),
                            )
                            // delete user account
                            .route(
                                "/user/{user_id}",
                                web::delete().to(admin_handlers::delete_user),
                            )
                            .service(
                                web::scope("/albums")
                                    //get all own albums
                                    .route("/", web::get().to(status))
                                    //get album be id
                                    .route("/{album_id}", web::get().to(status))
                                    //change album data (description or name)
                                    .route("/{album_id}", web::put().to(status))
                                    //delete own album by id
                                    .route("/{album_id}", web::delete().to(status))
                                    //delete photo from album
                                    .route(
                                        "/{album_id}/photos/{photo_id",
                                        web::delete().to(status),
                                    ),
                            ), //.route("/user/{id}", web::get().to(admin_handlers::get_user))
                    )
                    //user auth routes
                    .service(
                        web::scope("/user")
                            .wrap(my_identity_service::IdentityService::new(
                                cookie_factory_user,
                                pool.clone(),
                            ))
                            .route("/logout", web::post().to(logout))
                            .route("/me", web::get().to(handlers::get_user))
                            .route("/me", web::delete().to(handlers::delete_user))
                            //update only nickname
                            .route("/me", web::put().to(handlers::update_user))
                            //update password
                            .route("/me/password", web::put().to(status))
                            .service(
                                web::scope("/albums")
                                    //get all own albums
                                    .route("/", web::get().to(status))
                                    //create new album
                                    .route("/", web::post().to(status))
                                    //get own album by id
                                    .route("/{album_id}", web::post().to(status))
                                    //change album data (description or name)
                                    .route("/{album_id}", web::put().to(status))
                                    //add photos to album
                                    .route("/{album_id}", web::post().to(status))
                                    //delete own album
                                    .route("/{album_id}", web::delete().to(status))
                                    //delete own album
                                    .route(
                                        "/{album_id}/photos/{photo_id}",
                                        web::delete().to(status),
                                    ),
                            )
                            .service(
                                web::scope("/tag")
                                    //get 15 photos for tagging
                                    .route("/", web::get().to(status))
                                    //get own album by id
                                    .route("/{album_id}/action/{photo_id}", web::post().to(status))
                                    //verify tag
                                    .route("/{album_id}/action/{photo_id}", web::put().to(status)),
                            ),
                    )
                    .service(
                        web::scope("/albums")
                            //get albums for preview (all)
                            .route("/", web::get().to(status))
                            //get album by id
                            .route("/{album_id}", web::get().to(status))
                            //get photos from album (preview)
                            .route("/{album_id}/photos/{photo_id}", web::get().to(status)),
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
