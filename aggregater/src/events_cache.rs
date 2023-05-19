use std::{ops::DerefMut, sync::Mutex};

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
// use message::*;

// use crate::{message, nats::TypedNats};
use mbus_api::mbus_nats::{NatsMessageBus, Bus};
use mbus_api::message::EventMessage;
//use crate::events_store::*;
use crate::events_store_cr::*;
use k8s_openapi::api::core::v1::ConfigMap;

static CACHE: OnceCell<Mutex<Cache>> = OnceCell::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct EventStruct {
    pub volume: Volume,
    pub nexus: Nexus,
    pub pool: Pool,
    pub replica: Replica
}
impl Default for EventStruct {
    fn default() -> Self {
        EventStruct { 
            volume: Volume::default(), 
            nexus: Nexus::default(),
            pool: Pool::default(), 
            replica: Replica::default(),
        } 
     }
}

impl EventStruct {
    // pub fn from_event_store_data(init_data: CallHomeEvent) -> Self {
    //     let events = init_data.spec;
    //     EventStruct {
    //         volume: Volume {
    //             created: events.volume_events.created,
    //             deleted: events.volume_events.deleted
    //         },
    //         nexus: Nexus {
    //             created: events.nexus_events.created,
    //             deleted: events.nexus_events.deleted
    //         },
    //         pool: Pool {
    //             created: events.pool_events.created,
    //             deleted: events.pool_events.deleted
    //         },
    //         replica: Replica {
    //             created: events.replica_events.created,
    //             deleted: events.replica_events.deleted
    //         }
    //     }
    // }

    pub fn from_event_store_cm_data(init_data: ConfigMap) -> Self {
        serde_json::from_str(&init_data.data.unwrap().get("stats.json").unwrap()).unwrap()
    }

    pub fn from_event_store_data(init_data: UpdatedCallHomeEvent) -> Self {
        let data = init_data.spec;
        EventStruct {
            volume: Volume {
                created: *data.events.get(&EventCat::Volume).unwrap().get(&EventAc::Created).unwrap(),
                deleted: *data.events.get(&EventCat::Volume).unwrap().get(&EventAc::Deleted).unwrap()
            },
            nexus: Nexus {
                created: *data.events.get(&EventCat::Nexus).unwrap().get(&EventAc::Created).unwrap(),
                deleted: *data.events.get(&EventCat::Nexus).unwrap().get(&EventAc::Deleted).unwrap()
            },
            pool: Pool {
                created: *data.events.get(&EventCat::Pool).unwrap().get(&EventAc::Created).unwrap(),
                deleted: *data.events.get(&EventCat::Pool).unwrap().get(&EventAc::Deleted).unwrap()
            },
            replica: Replica {
                created: *data.events.get(&EventCat::Replica).unwrap().get(&EventAc::Created).unwrap(),
                deleted: *data.events.get(&EventCat::Replica).unwrap().get(&EventAc::Deleted).unwrap()
            }
        }
        //EventStruct { volume: Volume{created: events.volume_events.created, deleted: events.volume_events.deleted }, nexus: Nexus::default() } 
     }

    fn inc_counter(&mut self, category: &str, action: &str) -> Result<(), String> {
        match category {
            "volume" => self.volume.inc_counter(action),
            "nexus" => self.nexus.inc_counter(action),
            "pool" => self.pool.inc_counter(action),
            "replica" => self.replica.inc_counter(action),
            _ => {
                println!("{}", format!("invalid category name to get '{}'", category));
                Err("Invalid event category".to_string())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Volume { pub created: u32, pub deleted: u32 }
impl Default for Volume {
    fn default() -> Self {
        Volume {
            created: 0,
            deleted: 0,
        }
     }
}
impl Volume {

    fn inc_counter(&mut self, action: &str) -> Result<(), String> {
        match action {
            "created" => Ok(self.created+=1),
            "deleted" => Ok(self.deleted+=1),
            _ => {
                println!("{}", format!("invalid action name to get '{}'", action));
                Err("Invalid action for volume".to_string())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nexus { pub created: u32, pub deleted: u32 }
impl Default for Nexus {
    fn default() -> Self {
        Nexus {
            created: 0,
            deleted: 0,
        }
     }
}
impl Nexus {
    fn inc_counter(&mut self, action: &str) -> Result<(), String> {
        match action {
            "created" => Ok(self.created+=1),
            "deleted" => Ok(self.deleted+=1),
            _ => {
                println!("{}", format!("invalid action name to get '{}'", action));
                Err("Invalid action for nexus".to_string())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pool { pub created: u32, pub deleted: u32 }
impl Default for Pool {
    fn default() -> Self {
        Pool {
            created: 0,
            deleted: 0,
        }
     }
}
impl Pool {
    fn inc_counter(&mut self, action: &str) -> Result<(), String> {
        match action {
            "created" => Ok(self.created+=1),
            "deleted" => Ok(self.deleted+=1),
            _ => {
                println!("{}", format!("invalid action name to get '{}'", action));
                Err("Invalid action for pool".to_string())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Replica { pub created: u32, pub deleted: u32 }
impl Default for Replica {
    fn default() -> Self {
        Replica {
            created: 0,
            deleted: 0,
        }
     }
}
impl Replica {
    fn inc_counter(&mut self, action: &str) -> Result<(), String> {
        match action {
            "created" => Ok(self.created+=1),
            "deleted" => Ok(self.deleted+=1),
            _ => {
                println!("{}", format!("invalid action name to get '{}'", action));
                Err("Invalid action for replica".to_string())
            }
        }
    }
}

/// Cache to store data that has to be exposed though exporter.
pub struct Cache {
    events: EventStruct,
}

impl Cache {
    /// Initialize the cache with default value.
    pub fn initialize(events: EventStruct) {
        CACHE.get_or_init(|| Mutex::new(Self { events }));
    }

    /// Returns cache.
    pub fn get_cache() -> &'static Mutex<Cache> {
        CACHE.get().expect("Cache is not initialized")
    }

    /// Get data field in cache.
    pub fn data_mut(&mut self) -> &mut EventStruct {
        &mut self.events
    }
}

/// To store data in shared variable i.e cache.
pub async fn store_events(mut nats: NatsMessageBus) -> Result<(), String> {
    // Store events count
    let mut sub = nats.subscribe::<EventMessage>().await.map_err(|error| {
        println!("Error subscribing to jetstream: {:?}", error)
    }).unwrap();
    let mut count = 0;
    loop {
            if let Some(message) = sub.next().await
            {
                count += 1;
                println!(
                    "{}\t{:?}",
                    count,
                    message);
                let mut cache = match Cache::get_cache().lock() {
                    Ok(cache) => cache,
                    Err(error) => {
                        println!("Error while getting cache resource {}", error);
                        continue;
                    }
                };
                let events_cache = cache.deref_mut();
                events_cache.data_mut().inc_counter(message.category.as_str(), message.action.as_str())?;
                println!("Received event: {:?}", message);
                
            }
        }
}
