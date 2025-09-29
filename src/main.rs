use actix_web::http::StatusCode;
use actix_web::middleware::{ErrorHandlers, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use dotenv::dotenv;
use env_logger::Env;
use log::info;

mod database;
mod middleware;
mod utils;
use middleware::not_found::not_found;
use middleware::error_handler::handle_error;
use router::index::routes;
use serde_json::json;
mod router;
mod user;

#[get("/")]
async fn default() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Welcome to my Rust web-Server",
        "httpStatusCode": StatusCode::OK.as_u16(),
        "service": std::env::var("SERVICE_NAME").unwrap_or_else(|_| "Unknown".to_string()),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logger with environment variable support
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Log the server start
    info!("Starting server on http://localhost:8000");

    let mongo_client = database::connect_to_mongo()
        .await
        .expect("Failed to connect to MongoDB");

    // Create UserService
    // let user_service = web::Data::new(UserService::new(&mongo_client));
    // let todo_service = web::Data::new(TodoService::new(&mongo_client));

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(mongo_client.clone()))
            // .app_data(user_service.clone())
            // .app_data(todo_service.clone())
            .configure(routes)
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::NOT_FOUND, not_found)
                    .default_handler(handle_error),
            )
            .service(default)
    })
    .bind(("localhost", 8000))?
    .run()
    .await?;

    // Log after server has started (this line will only be reached when the server shuts down)
    info!("Server has stopped");

    Ok(())
}
