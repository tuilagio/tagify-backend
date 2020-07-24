use actix::{Actor, Addr, Arbiter, Context, System};
use goauth::auth::JwtClaims;
use goauth::scopes::Scope;
use goauth::credentials::Credentials;
use goauth::GoErr;
use goauth::get_token_as_string_legacy;
use smpl_jwt::Jwt;
use log::{error, debug};

pub struct MyActor;

impl Actor for MyActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        let credentials = Credentials::from_file("/home/lhebendanz/.config/gcloud/tagify-key.json").unwrap();
        let claims = JwtClaims::new(credentials.iss(),
                             &Scope::DevStorageReadWrite,
                             credentials.token_uri(),
                             None, None);
        let jwt = Jwt::new(claims, credentials.rsa_key().unwrap(), None);
        debug!("I am alive!");

        let token = match get_token_as_string_legacy(&jwt, Some(&credentials.token_uri())){
            Ok(i) => i,
            Err(e) => {
                error!("Error on get_token_as_string: {}",e);
                std::process::exit(2);
            }
        };
        println!("Token is {}", token);
        System::current().stop(); // <- stop system
    }
}
