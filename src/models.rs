use serde::{Serialize, Deserialize};
use tokio_pg_mapper_derive::PostgresMapper;
use log::{ error};

use actix_web::{
    Result,
};

use crate::errors::UserError;
// used for password hashing
use argon2::Config;
use rand::Rng;


#[derive(Debug,Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "users")]
pub struct User {
    pub username: String,
    pub nickname: String,
    pub password: String,
    pub is_admin: bool
}


#[derive(Debug,Serialize, Deserialize)]
pub struct UserData {
    pub username: String,
    pub password: String,
}

impl User {
    //create User instance without hasing password
    pub fn create_user(username: &String, password: &String, is_admin: bool) -> User {
        User {
            username: username.clone(),
            nickname: username.clone(),
            password: password.clone(),
            is_admin: is_admin
        }
    }

    pub fn hash_password(&mut self) -> Result<(), UserError> {
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = Config::default();

        self.password = match argon2::hash_encoded(self.password.as_bytes(), &salt, &config){
            Ok(item) => item,
            Err(e) => {
                error!("Error occured: {}",e );
                return Err(UserError::InternalError);
            }
        };
        Ok(())        
    }

    pub fn verify_password(&self, password: &[u8]) -> Result<bool, UserError> {
        match argon2::verify_encoded(&self.password, password){
            Ok(item) => item,
            Err(e) => {
                error!("Error occured: {}",e );
                return Err(UserError::InternalError);
            }
        };
        Ok(true)
    }
}