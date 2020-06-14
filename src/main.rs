#![cfg_attr(feature = "strict", deny(warnings))]

use actix_files as fs;
use actix_identity::IdentityService;
use actix_web::{middleware::Logger, web, App, HttpServer};

use listenfd::ListenFd;
use std::fs::File;
use std::io::Read;
use tokio_postgres::NoTls;

mod config;
mod db;
mod errors;
mod my_cookie_policy;
mod user_handlers;
mod album_handlers;
mod image_handlers;
mod img_meta_handlers;
mod utils;
mod models;

// use crate::admin_handlers::*;
// use crate::handlers::*;

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
    let mut server = HttpServer::new(move || {
        App::new()
            // Serve every file in directory from ../dist
            // .service(fs::Files::new("/app/debug_dist", "../debug_dist").show_files_listing())
            .data(pool.clone())
            /* Enable logger */
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                my_cookie_policy::MyCookieIdentityPolicy::new(cookie_key.as_bytes())
                    .name("auth-cookie")
                    .path("/")
                    .secure(false),
            ))
            // TODO: file size limit
            .data(web::JsonConfig::default().limit(4096))
            // .service(web::resource("/").route(web::get().to(status)))
            // .configure(routes)
            .service(
                web::scope("/api")
                    /* USER */
                    .service(
                        web::resource("/users").route(web::post().to(user_handlers::create_user))
                    )
                    .service(
                        web::scope("/users")
                            .service(
                                web::resource("/{user_id}")
                                    .route(web::delete().to(user_handlers::delete_user))
                                    .route(web::get().to(user_handlers::get_user))
                                    .route(web::put().to(user_handlers::update_user)),
                            )
                            .service(
                                web::resource("/{user_id}/albums")
                                    .route(web::get().to(user_handlers::get_user_albums)),
                            )
                    )
                    /* ALBUM */
                    .service(
                        web::resource("/albums").route(web::post().to(album_handlers::create_album_meta))
                    )
                    .service(
                        web::scope("/albums")
                            .service(
                                web::resource("/{album_id}")
                                    .route(web::delete().to(album_handlers::delete_album_meta))
                                    .route(web::get().to(album_handlers::get_album_meta))
                                    .route(web::put().to(album_handlers::update_album_meta)),
                            )
                            /* IMAGE */
                            .service(
                                web::resource("/{album_id}/images")
                                    .route(web::post().to(image_handlers::delete_all_images))
                            )
                            .service(
                                web::scope("/{album_id}/images")
                                    .service(
                                        web::resource("/{image_id}")
                                            .route(web::post().to(image_handlers::upload_image))
                                            .route(web::delete().to(image_handlers::delete_image))
                                            .route(web::get().to(image_handlers::get_image))
                                            .route(web::put().to(image_handlers::re_upload_image)),
                                    )
                            )
                            /* IMAGE-META */
                            .service(
                                web::resource("/{album_id}/image-metas")
                                    .route(web::post().to(img_meta_handlers::create_meta))
                                    .route(web::delete().to(img_meta_handlers::delete_all_metas))
                                    .route(web::get().to(img_meta_handlers::get_metas))
                            )
                            .service(
                                web::scope("/{album_id}/image-metas")
                                    .service(
                                        web::resource("/{img_metas_id}")
                                            .route(web::get().to(img_meta_handlers::get_meta))
                                            .route(web::delete().to(img_meta_handlers::delete_meta))
                                            .route(web::put().to(img_meta_handlers::update_meta)),
                                    )
                            )
                    )
                    /* AUTH */
                    .service(
                        web::scope("/auth")
                            .service(web::resource("/whoami").route(web::get().to(user_handlers::whoami)))
                            .service(web::resource("/login").route(web::post().to(user_handlers::login)))
                            .service(web::resource("/logout").route(web::post().to(user_handlers::logout)))
                    )
            )
    });
    /* Enables us to hot reload the server */
    let mut listenfd = ListenFd::from_env();
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind(ip)?
    };

    server.run().await
}
