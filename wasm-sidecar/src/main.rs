use std::{collections::HashMap, num::NonZeroU32, sync::Arc};

use governor::{Quota, RateLimiter};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use warp::{hyper::StatusCode, Filter};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,
    sub: String,
    exp: usize,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // GET / returns help
    let help = warp::get().and(warp::path::end()).map(|| {
        "Post form-data to / with token=J.W.T\n"
    });

    // POST / takes form with a token and performs rate limiting and openid verification on it.
    let q = Quota::per_minute(NonZeroU32::new(10).unwrap());
    let gov: Arc<RateLimiter<String, _, _>> = Arc::new(governor::RateLimiter::dashmap(q));

    let key: Arc<Hmac<Sha256>> = Arc::new(Hmac::new_from_slice(b"big secret").unwrap());

    let route = warp::post()
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .map(
            move |simple_map: HashMap<String, String>| match simple_map.get("token") {
                Some(token) => {
                    let claims: Result<Claims, _> = token.verify_with_key(key.as_ref());
                    match claims {
                        Ok(claims) => match gov.clone().check_key(&claims.sub) {
                            Ok(_) => StatusCode::OK,
                            Err(_) => StatusCode::TOO_MANY_REQUESTS,
                        },
                        Err(e) => {
                            println!("bad token {:#?}", e);
                            StatusCode::FORBIDDEN
                        }
                    }
                }
                None => StatusCode::UNAUTHORIZED,
            },
        );

    let routes = help.or(route);
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await
}
