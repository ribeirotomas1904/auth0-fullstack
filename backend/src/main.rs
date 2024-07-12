use std::{convert::Infallible, env};

use axum::{
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use dotenv::dotenv;
use http::{Method, Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

// TODO: Add more claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn authorize(mut req: Request, next: Next) -> Response<Body> {
    let domain = env::var("AUTH0_DOMAIN").unwrap();
    let audience = env::var("AUTH0_AUDIENCE").unwrap();

    let auth_header = req
        .headers_mut()
        .get(http::header::AUTHORIZATION)
        .unwrap()
        .to_str()
        .unwrap();

    let mut header = auth_header.split_whitespace();

    let jwks = reqwest::get(format!("https://{domain}/.well-known/jwks.json"))
        .await
        .unwrap()
        .json::<jsonwebtoken::jwk::JwkSet>()
        .await
        .unwrap();

    let (_, token) = (header.next(), header.next());
    let token = token.unwrap();

    let token_kid = jsonwebtoken::decode_header(token).unwrap().kid.unwrap();

    let jwk = jwks.find(&token_kid).unwrap();
    let decoding_key = DecodingKey::from_jwk(&jwk).unwrap();

    let mut validation = Validation::new(Algorithm::RS256);

    // TODO: Add more validation
    validation.set_audience(&[audience]);
    validation.set_issuer(&[format!("https://{domain}/")]);

    let token_data = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation).unwrap();

    req.extensions_mut().insert(token_data.claims.sub);
    next.run(req).await
}

pub async fn hello(Extension(user_sub): Extension<String>) -> impl IntoResponse {
    user_sub
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any)
        .allow_headers([http::header::AUTHORIZATION]);

    let app = Router::new().route(
        "/",
        get(hello)
            .layer::<_, Infallible>(middleware::from_fn(authorize))
            .layer(cors),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
