use actix::{Actor, Context};
use goauth::auth::JwtClaims;
use std::time::Duration;
use goauth::scopes::Scope;
use goauth::credentials::Credentials;
use goauth::get_token_as_string_legacy;
use smpl_jwt::Jwt;
use log::{error, debug};
use actix::prelude::AsyncContext;

const SECS_IN_MINUTE: u64 = 60;
pub struct MyActor;

pub fn get_token()-> String{
    let credentials = Credentials::from_file("/home/lhebendanz/.config/gcloud/tagify-key.json").unwrap();
    let claims = JwtClaims::new(credentials.iss(),
                         &Scope::DevStorageReadWrite,
                         credentials.token_uri(),
                         None, None);
    let jwt = Jwt::new(claims, credentials.rsa_key().unwrap(), None);
    debug!("Calling get_token");

    let token = match get_token_as_string_legacy(&jwt, Some(&credentials.token_uri())){
        Ok(i) => i,
        Err(e) => {
            error!("Error on get_token_as_string: {}",e);
            std::process::exit(2);
        }
    };
    return token;
}

impl Actor for MyActor {
    type Context = Context<Self>;


    fn started(&mut self, ctx: &mut Self::Context) {

        let mut token = get_token();
        debug!("Token is {}", token);

        ctx.run_interval(Duration::new(15*SECS_IN_MINUTE, 0), move |_act, _ctx| {
            token = get_token();
            debug!("Token is {}", token);
        });

    }
}
