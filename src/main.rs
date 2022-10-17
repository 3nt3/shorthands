use std::fs;

use actix_web::{
    get, http::StatusCode, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
};
use serde::{Deserialize, Serialize};

#[get("/")]
async fn handle_everything(_req: HttpRequest) -> HttpResponse {
    let host = _req.headers().get("host").unwrap().to_str().unwrap();
    println!("Host: {:?}", host);

    let domain_parts: Vec<&str> = host.split('.').collect();
    let subdomain = domain_parts[0];

    if domain_parts.len() < 3 {
        // this should honestly never happen because nginx only routes subdomains
        return HttpResponseBuilder::new(StatusCode::BAD_REQUEST).body("no subdomain");
    }

    println!("subdomain: {:?}", subdomain);

    match read_shorthands() {
        Ok(shorthands) => {
            if subdomain == "list" {
                return HttpResponse::Ok()
                    .insert_header(("Content-Type", "application/json"))
                    .json(shorthands);
            }

            match shorthands.iter().find(|&x| x.short == subdomain) {
                Some(shorthand) => {
                    println!("Redirecting {} to {}", host, shorthand.long);
                    return HttpResponseBuilder::new(StatusCode::FOUND)
                        .insert_header(("Location", shorthand.long.clone()))
                        .body(format!("Redirecting to: {}", shorthand.long));
                }
                None => HttpResponseBuilder::new(StatusCode::NOT_FOUND).body("Not found"),
            }
        }
        Err(err) => {
            println!("Failed getting shorthands: {err}");
            return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Internal server error: {}", err.to_string()));
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Shorthand {
    short: String,
    long: String,
}

fn read_shorthands() -> anyhow::Result<Vec<Shorthand>> {
    let contents = fs::read_to_string("./shorthands.json")?;
    match serde_json::from_str::<Vec<Shorthand>>(&contents) {
        serde_json::Result::Ok(x) => {
            // check for 'list' shorthand
            match x.iter().find(|item| item.short == "list") {
                Some(_) => return Err(anyhow::Error::msg("The shorthand 'list' is reserverd.")),
                None => return anyhow::Ok(x),
            }
        }
        serde_json::Result::Err(err) => Err(anyhow::Error::msg(err.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match read_shorthands() {
        Ok(_) => println!("Shorthands seem to exist and are able to be read :)"),
        Err(err) => panic!("Error reading shorthands: {}", err),
    }

    HttpServer::new(|| App::new().service(handle_everything))
        .bind(("127.0.0.1", 8086))?
        .run()
        .await
}
