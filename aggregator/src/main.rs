use crate::{
    common::{constants::MBUS_URL, error},
    events::events_store::initialize,
};
use k8s_openapi::api::core::v1::ConfigMap;
use events::events_collector::StatsCollector;
use events::events_cache::{Cache, EventSet};
use mbus_api::mbus_nats::{message_bus_init, NatsMessageBus};
use tracing::{error, info, warn};
use utils::{
    raw_version_str,
    tracing_telemetry::{default_tracing_tags, flush_traces, init_tracing},
};
use actix_web::{http::header, middleware, web, HttpResponse, HttpServer, Responder};
use prometheus::{Encoder, Registry};


mod common;
mod events;
mod exporter;

async fn mbus_init() -> error::Result<NatsMessageBus> {
    let message_bus = message_bus_init(MBUS_URL).await;
    Ok(message_bus)
}

/// Initialize events store cache from config map.
async fn initialize_events_cache(init_data: ConfigMap) {
    let events = EventSet::from_event_store(init_data).unwrap();
    Cache::initialize(events);
}

/// Initialize events store.
async fn initialize_events_store(namespace: &str) -> error::Result<ConfigMap> {
    let event_store = initialize(namespace).await?;
    Ok(event_store)
}

#[tokio::main]
async fn main() -> error::Result<()> {
    init_logging();
    info!("stats aggregation started!");
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    let m_bus = mbus_init().await?;
    info!("mbus initialized successfully!");
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    let init_data = initialize_events_store("mayastor").await?;
    info!("event store initialized successfully!");
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    
    initialize_events_cache(init_data).await;
    info!("event cache initialized successfully!");
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    // spawn a new task to store the data in cache.
    tokio::spawn(async move {
        events::events_cache::store_events(m_bus)
            .await
            .map_err(|error| {
                error!(%error, "Error while storing the events to cahce");
                flush_traces();
                error
            })
    });

    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    // spawn a new task to update the config map from cache.

    tokio::spawn(async move {
        events::events_store::update_config_map_data("mayastor")
            .await
            .map_err(|error| {
                error!(%error, "Error while persisting the stats to config map from cache");
                flush_traces();
                error
            })
    });

    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    let app = move || {
        actix_web::App::new()
            .wrap(middleware::Logger::default())
            .configure(stats_route)
    };


    HttpServer::new(app)
    .bind(exporter::exporter_config::ExporterConfig::get_config().metrics_endpoint())
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

/// Initialize logging components -- tracing.
fn init_logging() {
    let tags = default_tracing_tags(raw_version_str(), env!("CARGO_PKG_VERSION"));
    init_tracing("stats", tags, None);
}
