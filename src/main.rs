use std::fs;

use actix_web::{
    get, http::StatusCode, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
};
use serde::Deserialize;
use url::Url;

#[get("/")]
async fn handle_everything(_req: HttpRequest) -> HttpResponse {
    let host_str = _req.headers().get("host").unwrap().to_str().unwrap();
    println!("host_str: {:?}", host_str);

    // not sure if it's really necessary to parse the url but I guess at least the host has to be
    // valid this way
    let host = Url::parse(host_str).unwrap().to_string();

    let domain_parts: Vec<&str> = host.split('.').collect();
    if domain_parts.len() < 3 {
        return HttpResponseBuilder::new(StatusCode::BAD_REQUEST).body("no subdomain provided");
    }
    let subdomain = domain_parts[0];

    println!("subdomain: {:?}", subdomain);

    match read_shorthands() {
        Ok(shorthands) => match shorthands.iter().find(|&x| x.short == subdomain) {
            Some(shorthand) => HttpResponseBuilder::new(StatusCode::FOUND)
                .insert_header(("Location", "TODO"))
                .body(format!("redirecting to: {}", shorthand.long)),
            None => HttpResponseBuilder::new(StatusCode::NOT_FOUND).body("not found"),
        },
        Err(err) => {
            println!("failed getting shorthands: {err}");
            return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("internal server error: {}", err.to_string()));
        }
    }
}

#[derive(Deserialize)]
struct Shorthand {
    short: String,
    long: String,
}

fn read_shorthands() -> anyhow::Result<Vec<Shorthand>> {
    let contents = fs::read_to_string("./shorthands.json")?;
    match serde_json::from_str(&contents) {
        serde_json::Result::Ok(x) => anyhow::Ok(x),
        serde_json::Result::Err(err) => Err(anyhow::Error::msg(err.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(handle_everything))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
