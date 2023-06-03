mod events_cache;
use events_collector::StatsCollector;
use events_cache::*;
use exporter_config::ExporterConfig;
// use nats_connection::NatsConnectionSpec;
use mbus_api::mbus_nats::{message_bus_init, NatsMessageBus};
use std::{thread};
mod message;
mod nats;
mod nats_connection;
mod retry;
mod exporter_config;
mod events_collector;

// mod events_store_cm;
// use events_store_cm::*;
mod events_store_cr;
use events_store_cr::*;

use actix_web::{http::header, middleware, web, HttpResponse, HttpServer, Responder};
use prometheus::{Encoder, Registry};
use tracing::{error, warn};
use std::time::Duration;
use k8s_openapi::api::core::v1::ConfigMap;

// Initialize exporter config that are passed through arguments
fn initialize_exporter() {
    ExporterConfig::initialize();
}

/// Initialize cache
async fn initialize_events_cache(init_data: UpdatedCallHomeEvent) {
    Cache::initialize(EventStruct::from_event_store_data(init_data));
}

/// Initialize events_store
async fn initialize_events_store() -> UpdatedCallHomeEvent {
    initialize_cr().await.unwrap()
}

async fn mbus_init() -> NatsMessageBus {
    // NatsMessageBus::new("nats://localhost:4222").await.subscribe().await.unwrap()
    message_bus_init("nats://mayastor-nats:4222").await
}

// /// Initialize cache from cm
// async fn initialize_events_cache(init_data: ConfigMap) {
//     Cache::initialize(EventStruct::from_event_store_cm_data(init_data));
// }

// /// Initialize events_store_cm
// async fn initialize_events_store_cm() -> ConfigMap {
//     initialize_configmap().await.unwrap()
// }


#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    // let nats = NatsConnectionSpec::from_url("nats://mayastor-nats:4222")?
    //     .connect()
    //     .await?;
    let nats = mbus_init().await;
    let init_data = initialize_events_store().await;
    initialize_events_cache(init_data).await;
    initialize_exporter();

    tokio::spawn(async move {
        events_cache::store_events(nats)
            .await
            .expect("Unable to store data in events cache");
    }); 
    tokio::spawn(async move {
        update_cr()
            .await
            .expect("Unable to update the cr");
    }); 
    let app = move || {
        actix_web::App::new()
            .wrap(middleware::Logger::default())
            .configure(stats_route)
    };

    HttpServer::new(app)
        .bind(ExporterConfig::get_config().metrics_endpoint())
        .unwrap()
        .run()
        .await
        .expect("Port should be free to expose the stats");
    Ok(())
}

fn stats_route(cfg: &mut web::ServiceConfig) {
    cfg.route("/stats", web::get().to(metrics_handlers));
}


async fn metrics_handlers() -> impl Responder {
    // Initialize stats collector
    let stats_collector = StatsCollector::default();
    // Create a new registry for prometheus
    let registry = Registry::default();
    // Register stats collector in the registry
    if let Err(error) = Registry::register(&registry, Box::new(stats_collector)) {
        warn!(%error, "Stats collector already registered");
    }
    let mut buffer = Vec::new();

    let encoder = prometheus::TextEncoder::new();
    // Starts collecting metrics via calling gatherers
    if let Err(error) = encoder.encode(&registry.gather(), &mut buffer) {
        error!(%error, "Could not encode custom metrics");
    };

    let res_custom = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(error) => {
            error!(%error, "Prometheus metrics could not be parsed from_utf8'd");
            String::default()
        }
    };
    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_PLAIN))
        .body(res_custom)
}
