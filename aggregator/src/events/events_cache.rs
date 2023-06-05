use std::{ops::DerefMut, sync::Mutex};

use crate::common::{constants::EVENT_STATS_DATA, error};
use k8s_openapi::api::core::v1::ConfigMap;
use mbus_api::{
    mbus_nats::NatsMessageBus,
    message::{Action, Category, EventMessage},
    Bus,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
static CACHE: OnceCell<Mutex<Cache>> = OnceCell::new();

/// EventSet captures the type of events.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EventSet {
    pub pool: Pool,
    pub volume: Volume,
}

impl EventSet {
    pub fn from_event_store(init_data: ConfigMap) -> error::Result<Self> {
        let data = init_data
            .data
            .ok_or(error::ReferenceConfigMapNoData.build())?;
        let value = data.get(EVENT_STATS_DATA).ok_or(
            error::ReferencedKeyNotPresent {
                key: EVENT_STATS_DATA.to_string(),
            }
            .build(),
        )?;

        let event_set = serde_json::from_str(value)
            .context(error::EventSerdeDeserialization { event: value })?;
        Ok(event_set)
    }

    fn inc_counter(&mut self, category: Category, action: Action) -> error::Result<()> {
        match category {
            Category::Pool => self.pool.inc_counter(action),
            Category::Volume => self.volume.inc_counter(action),
        }
    }
}

impl From<&mut EventSet> for EventSet {
    fn from(event_set: &mut EventSet) -> Self {
        EventSet {
            pool: event_set.pool.clone(),
            volume: event_set.volume.clone(),
        }
    }
}

/// Volume related events.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Volume {
    pub volume_created: u32,
    pub volume_deleted: u32,
}

impl Volume {
    fn inc_counter(&mut self, action: Action) -> error::Result<()> {
        match action {
            Action::CreateEvent => {
                self.volume_created += 1;
            }
            Action::DeleteEvent => {
                self.volume_deleted += 1;
            }
        }
        Ok(())
    }
}

/// Pool related events.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Pool {
    pub pool_created: u32,
    pub pool_deleted: u32,
}

impl Pool {
    fn inc_counter(&mut self, action: Action) -> error::Result<()> {
        match action {
            Action::CreateEvent => {
                self.pool_created += 1;
            }
            Action::DeleteEvent => {
                self.pool_deleted += 1;
            }
        }
        Ok(())
    }
}

/// Cache to store data that has to be exposed though exporter.
pub struct Cache {
    events: EventSet,
}

impl Cache {
    /// Initialize the cache with default value.
    pub fn initialize(events: EventSet) {
        CACHE.get_or_init(|| Mutex::new(Self { events }));
    }

    /// Returns cache.
    pub fn get_cache() -> &'static Mutex<Cache> {
        CACHE.get().expect("Cache is not initialized")
    }

    /// Get data field in cache.
    pub fn data_mut(&mut self) -> &mut EventSet {
        &mut self.events
    }
}

/// To store data in shared variable i.e cache.
pub async fn store_events(mut nats: NatsMessageBus) -> error::Result<()> {
    let mut sub = nats
        .subscribe::<EventMessage>()
        .await
        .map_err(|error| println!("Error subscribing to jetstream: {:?}", error))
        .unwrap();

    loop {
        if let Some(message) = sub.next().await {
            let mut cache = match Cache::get_cache().lock() {
                Ok(cache) => cache,
                Err(error) => {
                    println!("Error while getting cache resource {}", error);
                    continue;
                }
            };
            let events_cache = cache.deref_mut();
            events_cache
                .data_mut()
                .inc_counter(message.category, message.action)?;
        }
    }
}
