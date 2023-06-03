use std::{ops::DerefMut, time::Duration};


use anyhow::{Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{Api, PostParams, PatchParams, Patch},
    core::crd::CustomResourceExt,
    Client, CustomResource, runtime::wait::{await_condition, conditions},
};
use crate::events_cache::Cache;


// Own custom resource
#[derive(CustomResource, Deserialize, Serialize, Eq, PartialEq, Clone, Debug, JsonSchema)]
#[kube(
group = "openebs.io", 
version = "v1alpha1", 
kind = "CallHomeEvent", 
plural = "callhomeevents",
namespaced,  
derive = "PartialEq",
derive = "Default",
)]
pub struct CallHomeEventSpec {
    pub volume_events: VolumeEvents,
    pub nexus_events: NexusEvents,
    pub pool_events: PoolEvents,
    pub replica_events: ReplicaEvents,
}


impl Default for CallHomeEventSpec {
    fn default() -> Self {
        CallHomeEventSpec {
            nexus_events: NexusEvents::default(),
            volume_events: VolumeEvents::default(),
            pool_events: PoolEvents::default(),
            replica_events: ReplicaEvents::default(),
        }
    }
}


#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, Default, JsonSchema)]
pub struct VolumeEvents {
    pub created: u32,
    pub deleted: u32,
}


#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, Default, JsonSchema)]
pub struct NexusEvents {
    pub created: u32,
    pub deleted: u32,
}


#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, Default, JsonSchema)]
pub struct PoolEvents {
    pub created: u32,
    pub deleted: u32,
}


#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, Default, JsonSchema)]
pub struct ReplicaEvents {
    pub created: u32,
    pub deleted: u32,
}


pub async fn initialize_cr() -> Result<CallHomeEvent, kube::Error> {
    let client = Client::try_default().await.unwrap();
    let api = Api::<CustomResourceDefinition>::all(client.clone());
    if api.get("callhomeevents.openebs.io").await.is_ok() {
        println!("crd already exists");
    }
    else {
        let crd = create_crd(client.clone()).await;
        println!("crd: {:?}", crd);
    }
    let api1 = Api::<CallHomeEvent>::namespaced(client.clone(), "mayastor");
    if api1.get("callhome").await.is_ok() {
        println!("cr already exists");
    }
    else {
        let cr = create_cr(client.clone()).await;;
        println!("cr: {:?}", cr);


    }
    api1.get("callhome").await
}


// Create CRD and wait for it to be ready.
async fn create_crd(client: Client) -> Result<CustomResourceDefinition> {
    let api = Api::<CustomResourceDefinition>::all(client);
    let crd = CallHomeEvent::crd();
    println!(
        "Creating CRD: {}",
        serde_json::to_string_pretty(&crd).unwrap()
    );
    api.create(&PostParams::default(), &crd).await?;


    // Wait until it's accepted and established by the api-server
    println!("Waiting for the api-server to accept the CRD");
    let establish = await_condition(api.clone(), "callhomeevents.openebs.io", conditions::is_crd_established());
    let _ = tokio::time::timeout(std::time::Duration::from_secs(10), establish).await?;


    // It's served by the api - get it and return it
    let crd = api.get("callhomeevents.openebs.io").await?;
    Ok(crd)
}


// Create CRD and wait for it to be ready.
async fn create_cr(client: Client) {
    let api = Api::<CallHomeEvent>::namespaced(client.clone(), "mayastor");
    let f1 = CallHomeEvent::new("callhome", CallHomeEventSpec {
        nexus_events: NexusEvents {
            created: 0,
            deleted: 0,
        },
        volume_events: VolumeEvents {
            created: 0,
            deleted: 0,
        },
        pool_events: PoolEvents {
            created: 0,
            deleted: 0,
        },
        replica_events: ReplicaEvents {
            created: 0,
            deleted: 0,
        },


    });
    let o = api.create(&PostParams::default(), &f1).await.unwrap();
}


pub async fn update_cr() -> Result<(), String> {
    let client = Client::try_default().await.unwrap();
    let api: Api<CallHomeEvent> = Api::namespaced(client.clone(), "mayastor");
    loop {
        let f1 = get_updated_events();
        let ssapply = PatchParams::apply("events_store").force();
        api.patch("callhome", &ssapply, &Patch::Apply(&f1)).await.unwrap();
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}


fn get_updated_events() -> CallHomeEvent {
    let mut c = Cache::get_cache().lock().unwrap();
    let mut binding = c.deref_mut().data_mut();
    let cp = binding.deref_mut();
    CallHomeEvent::new("callhome", CallHomeEventSpec {
        nexus_events: NexusEvents {
            created: cp.nexus.created,
            deleted: cp.nexus.deleted,
        },
        volume_events: VolumeEvents {
            created: cp.volume.created,
            deleted: cp.volume.deleted,
        },
        pool_events: PoolEvents {
            created: cp.pool.created,
            deleted: cp.pool.deleted,
        },
        replica_events: ReplicaEvents {
            created: cp.replica.created,
            deleted: cp.replica.deleted,
        },    
    })
}
