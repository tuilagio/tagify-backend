use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

// used for password hashing
use argon2::Config;
use rand::Rng;

#[derive(Serialize)]
pub struct Status {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "users")]
pub struct User {
    pub username: String,
    pub nickname: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedUser {
    pub username: String,
    pub password: String,
    pub repeat_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedLoginData {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nickname {
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Password {
    pub password: String,
    pub repeat_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendUser {
    pub username: String,
    pub nickname: String,
    pub is_admin: bool,
}

impl User {
    //create User instance without hasing password
    pub fn create_user(username: &str, password: &str, is_admin: bool) -> User {
        User {
            username: String::from(username),
            nickname: String::from(username),
            password: String::from(password),
            is_admin: is_admin,
        }
    }
}



// Hash password, can be implemented for Structs containing .passwort attribut
pub trait Hash {
     fn hash_password(&mut self) -> Result<(), argon2::Error>;

    #[allow(dead_code)]
     fn verify_password(&self, password: &[u8]) -> Result<bool, argon2::Error>;
}

//Hash implementation for User & password in one Trait

impl Hash for User {
    fn hash_password(&mut self) -> Result<(), argon2::Error>{
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = Config::default();

        self.password = argon2::hash_encoded(self.password.as_bytes(), &salt, &config)?;
        Ok(())
    }

    #[allow(dead_code)]
     fn verify_password(&self, password: &[u8]) -> Result<bool, argon2::Error> {
        Ok(argon2::verify_encoded(&self.password, password)?)
    }
}

impl Hash for Password {
    fn hash_password(&mut self) -> Result<(), argon2::Error>{
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = Config::default();

        self.password = argon2::hash_encoded(self.password.as_bytes(), &salt, &config)?;
        Ok(())
    }

    #[allow(dead_code)]
     fn verify_password(&self, password: &[u8]) -> Result<bool, argon2::Error> {
        Ok(argon2::verify_encoded(&self.password, password)?)
    }
}