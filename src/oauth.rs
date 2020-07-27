use actix::{Actor, Context};
use goauth::auth::JwtClaims;
use std::io::Write;
use std::time::Duration;
use goauth::scopes::Scope;
use goauth::credentials::Credentials;
use goauth::get_token_as_string_legacy;
use smpl_jwt::Jwt;
use log::{error, debug};
use actix::prelude::AsyncContext;

const SECS_IN_MINUTE: u64 = 60;
pub struct Oauth {
    pub cred_file: String,
    pub key_file: String,
}

impl Oauth {
    pub fn new(cred_file: &str, key_file: &str) -> Self {
        return Self {
            cred_file: cred_file.to_string(),
            key_file: key_file.to_string()
        }
    }

    pub fn get_token(&self)-> String {
        let credentials = Credentials::from_file(&self.cred_file).expect("Could not read google credential file");
        let claims = JwtClaims::new(credentials.iss(),
                             &Scope::DevStorageReadWrite,
                             credentials.token_uri(),
                             None, None);
        let jwt = Jwt::new(claims, credentials.rsa_key().expect("Invalid rsa key in credential file"), None);
        debug!("Calling get_token");

        let token = match get_token_as_string_legacy(&jwt, Some(&credentials.token_uri())){
            Ok(i) => i,
            Err(e) => {
                error!("Error on get_token_as_string: {}",e);
                std::process::exit(2);
            }
        };

        let json: serde_json::Value = serde_json::from_str(&token).expect("JSON TokenResponse was not well-formatted");
        let token_bearer: String = json.get("access_token").expect("TokenResponse should have 'access_token' key").to_string().replace("\"", "");

        return token_bearer;
    }

}

impl Actor for Oauth {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        let mut token = self.get_token();

        let mut file = std::fs::File::create(&self.key_file).expect("key_file creation failed");
        file.write_all(token.as_bytes()).expect("oauth write failed");
        debug!("Token is {} write to {}", token, &self.key_file);

        ctx.run_interval(Duration::new(20*SECS_IN_MINUTE, 0), move |_act, _ctx| {
            let mut file = std::fs::File::create(_act.key_file.clone()).expect("key_file creation failed");
            token = _act.get_token();
            file.write_all(token.as_bytes()).expect("oauth write failed");
            debug!("Token is {}", token);
        });

    }

    // fn started(&mut self, ctx: &mut Self::Context) {

    //     let mut token = self.get_token();
    //     let mut file = std::fs::File::create(&self.key_file).expect("key_file creation failed");
    //     file.write_all(token.as_bytes()).expect("oauth write failed");
    //     debug!("Token is {}", token);

    //     ctx.run_interval(Duration::new(15*SECS_IN_MINUTE, 0), move |_act, _ctx| {
    //         token = _act.get_token();
    //         file.write_all(token.as_bytes()).expect("oauth write failed");
    //         debug!("Token is {}", token);
    //     });

    // }
}
