use actix_files::NamedFile;
use actix_web::{
    Result,
    get,
    put,
    delete,
    HttpResponse,
    ResponseError,
    HttpServer,
    App,
    web,
    middleware::Logger,
    http::header,
};
use actix_files as fs;
use std::path::PathBuf;
use listenfd::ListenFd;
use tokio_postgres::{NoTls};
use actix_http::ResponseBuilder;
use std::fs::File;
use actix_web::http::StatusCode;
use std::io::Read;
use serde::{Serialize, Deserialize};
use deadpool_postgres::{Pool};
use tokio_pg_mapper_derive::PostgresMapper;
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use tokio_pg_mapper::FromTokioPostgresRow;
use failure::Fail;
use log::{debug, error};

#[derive(Deserialize, Clone)]
struct Server {
    pub hostname: String,
    pub port: String,
    pub key: String,
}

#[derive(Deserialize, Clone)]
struct MyConfig {
    pub postgres: deadpool_postgres::Config,
    pub server: Server
}

impl MyConfig {
    pub fn new(path: &str) -> Result<Self, config::ConfigError>{
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name(path)).unwrap();
        settings.try_into()
    }
}

#[derive(Debug,Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "todos")]
struct Todo {
    id: Option<i32>,
    description: String,
    date: String,
    progress: i32
}

#[derive(Fail, Debug)]
enum UserError {
    #[fail(display = "Parsing error on field: {}", field)]
    BadClientData { field: String  },
    #[fail(display = "An internal error occured. Try again later")]
    InternalError,
    #[fail(display = "You are not logged in")]
    AuthFail,
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::BadClientData {..} => StatusCode::BAD_REQUEST,
            UserError::AuthFail => StatusCode::UNAUTHORIZED,
        }
    }
}

#[get("/")]
async fn index() -> Result<NamedFile> {
    let path: PathBuf = PathBuf::from("../debug_dist/index.html");
    Ok(NamedFile::open(path)?)
}


#[put("/add_todo")]
async fn add_todo(pool: web::Data<Pool>, data: web::Json<Todo>) -> Result<HttpResponse, UserError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}",e );
            return Err(UserError::InternalError);
        }
    };

    debug!("{:#?}", data);

    // Query data
    let result = client.execute("INSERT INTO todos (description, date, progress) VALUES ($1,$2,$3)", &[&data.description, &data.date, &data.progress]).await;

    if let Err(e) = result {
        error!("Error occured: {}",e );
        return Err(UserError::InternalError);
    }

    Ok(HttpResponse::new(StatusCode::OK))
}

#[delete("/delete_todo/{id}")]
async fn delete_todo(pool: web::Data<Pool>, data: web::Path<(i32,)>) -> Result<HttpResponse, UserError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}",e );
            return Err(UserError::InternalError);
        }
    };

    debug!("Todo id tos delete: {}", data.0);

    // Query data
    let result = client.execute("DELETE FROM todos WHERE id = $1", &[&data.0]).await;

    match result {
        Err(e) => {
            error!("Error occured: {}",e );
            return Err(UserError::InternalError);
        },
        Ok(num_updated) => {
            if num_updated == 0 {
                return Err(UserError::BadClientData{field: "id does not exist".to_string()  });
            }
        }
    };


    Ok(HttpResponse::new(StatusCode::OK))
}

#[put("/update_todo")]
async fn update_todo(pool: web::Data<Pool>, data: web::Json<Todo>) -> Result<HttpResponse, UserError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}",e );
            return Err(UserError::InternalError);
        }
    };

    debug!("{:#?}", data);

    let id = match data.id {
        Some(i) => i,
        None => return Err(UserError::BadClientData{field: "id is missing".to_string()})
    };

    // Query data
    let result = client.execute("UPDATE todos SET description = $1, date = $2, progress = $3 WHERE id = $4", &[&data.description, &data.date,&data.progress, &id])
        .await;


    match result {
        Err(e) => {
            error!("Error occured: {}",e );
            return Err(UserError::InternalError);
        },
        Ok(num_updated) => {
            if num_updated == 0 {
                return Err(UserError::BadClientData{field: "id does not exist".to_string()  });
            }
        }
    };

    Ok(HttpResponse::new(StatusCode::OK))
}

#[put("login")]
async fn login(pool: web::Data<Pool>, id: Identity) -> Result<HttpResponse, UserError> {
    id.remember("User1".to_owned());

    Ok(HttpResponse::new(StatusCode::OK))
}

#[put("logout")]
async fn logout(id: Identity) -> Result<HttpResponse, UserError> {
    id.forget();

    Ok(HttpResponse::new(StatusCode::OK))
}

#[get("get_user")]
async fn get_user(id: Identity) -> Result<String, UserError> {
    // access request identity
    if let Some(id) = id.identity() {
        Ok(format!("Welcome! {}", id))
    } else {
        Err(UserError::AuthFail)
    }
}

#[get("/get_all_todos")]
async fn get_all_todos(pool: web::Data<Pool>) -> Result<HttpResponse, UserError> {

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}",e );
            return Err(UserError::InternalError);
        }
    };

    // Query data
    let res = match client.query("SELECT * from todos", &[]).await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}",e );
            return Err(UserError::InternalError);
        }
    };

    let mut body: Vec<Todo> = Vec::new();

    // Serialize data
    for row in res {
        let result = Todo::from_row_ref(&row);

        let todo = match result {
            Err(e) => {
                error!("Error occured: {}",e );
                return Err(UserError::InternalError);
            },
            Ok(todo) => todo
        };
        body.push(todo);
    };

    Ok(HttpResponse::build(StatusCode::OK).json(body))
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    // Read config
    let conf = MyConfig::new("Settings").unwrap();

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

    let cookie_key = conf.server.key;
    // Register http routes
    let mut server = HttpServer::new(move|| {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(cookie_key.as_bytes())
                .name("auth-cookie")
                .secure(false)
            ))
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

            // Login handlers
            .service(login)
            .service(get_user)
            .service(logout)
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
