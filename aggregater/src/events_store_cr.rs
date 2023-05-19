use std::{ops::DerefMut, time::Duration, collections::HashMap};

use anyhow::Result;
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
kind = "UpdatedCallHomeEvent", 
plural = "updatedcallhomeevents",
namespaced,  
derive = "PartialEq",
derive = "Default",
)]
pub struct UpdatedCallHomeEventSpec {
    pub events: HashMap<EventCat, HashMap<EventAc, u32>>
}

impl Default for UpdatedCallHomeEventSpec {
    fn default() -> Self {
        UpdatedCallHomeEventSpec {
            events: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, JsonSchema)]
pub enum EventCat {
    Volume,
    Replica,
    Pool,
    Nexus,
    Test,
    }
 
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, JsonSchema)]
pub enum EventAc {
    Created,
    Deleted,
    Changed,
}

pub async fn initialize_cr() -> Result<UpdatedCallHomeEvent, kube::Error> {
    let client = Client::try_default().await.unwrap();
    let api = Api::<CustomResourceDefinition>::all(client.clone());
    if api.get("updatedcallhomeevents.openebs.io").await.is_ok() {
        println!("crd already exists");
    }
    else {
        let crd = create_crd(client.clone()).await;
        println!("crd: {:?}", crd);
    }
    let api1 = Api::<UpdatedCallHomeEvent>::namespaced(client.clone(), "mayastor");
    if api1.get("updatedcallhome").await.is_ok() {
        println!("cr already exists");
    }
    else {
        let cr = create_cr(client.clone()).await;;
        println!("cr: {:?}", cr);
        
    }
    api1.get("updatedcallhome").await
}

// Create CRD and wait for it to be ready.
async fn create_crd(client: Client) -> Result<CustomResourceDefinition> {
    let api = Api::<CustomResourceDefinition>::all(client);
    let crd = UpdatedCallHomeEvent::crd();
    println!(
        "Creating CRD: {}",
        serde_json::to_string_pretty(&crd).unwrap()
    );
    api.create(&PostParams::default(), &crd).await?;

    // Wait until it's accepted and established by the api-server
    println!("Waiting for the api-server to accept the CRD");
    let establish = await_condition(api.clone(), "updatedcallhomeevents.openebs.io", conditions::is_crd_established());
    let _ = tokio::time::timeout(std::time::Duration::from_secs(10), establish).await?;

    // It's served by the api - get it and return it
    let crd = api.get("updatedcallhomeevents.openebs.io").await?;
    Ok(crd)
}

// Create CRD and wait for it to be ready.
async fn create_cr(client: Client) {
    let api = Api::<UpdatedCallHomeEvent>::namespaced(client.clone(), "mayastor");
    let f1 = get_init_events();
    let o = api.create(&PostParams::default(), &f1).await.unwrap();
}

pub async fn update_cr() -> Result<(), String> {
    let client = Client::try_default().await.unwrap();
    let api: Api<UpdatedCallHomeEvent> = Api::namespaced(client.clone(), "mayastor");
    loop {
        let f1 = get_updated_events();
        let ssapply = PatchParams::apply("events_store").force();
        api.patch("updatedcallhome", &ssapply, &Patch::Apply(&f1)).await.unwrap();
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

fn get_init_events() -> UpdatedCallHomeEvent {

    let mut volumedata = HashMap::new();
    volumedata.insert(EventAc::Created, 0);
    volumedata.insert(EventAc::Deleted, 0);
    let mut pooldata = HashMap::new();
    pooldata.insert(EventAc::Created, 0);
    pooldata.insert(EventAc::Deleted, 0);
    let mut replicadata = HashMap::new();
    replicadata.insert(EventAc::Created, 0);
    replicadata.insert(EventAc::Deleted, 0);
    let mut nexusdata = HashMap::new();
    nexusdata.insert(EventAc::Created, 0);
    nexusdata.insert(EventAc::Deleted, 0);
    nexusdata.insert(EventAc::Changed, 0);
    let mut testdata = HashMap::new();
    nexusdata.insert(EventAc::Created, 0);

    let mut callhomedata = HashMap::new();
    callhomedata.insert(EventCat::Volume, volumedata);
    callhomedata.insert(EventCat::Pool, pooldata);
    callhomedata.insert(EventCat::Replica, replicadata);
    callhomedata.insert(EventCat::Nexus, nexusdata);
    callhomedata.insert(EventCat::Test, testdata);
    UpdatedCallHomeEvent::new("updatedcallhome", UpdatedCallHomeEventSpec {
        events: callhomedata
    })
}

fn get_updated_events() -> UpdatedCallHomeEvent {

    let mut c = Cache::get_cache().lock().unwrap();
    let mut binding = c.deref_mut().data_mut();
    let cp = binding.deref_mut();

    let mut volumedata = HashMap::new();
    volumedata.insert(EventAc::Created, cp.volume.created);
    volumedata.insert(EventAc::Deleted, cp.volume.deleted);
    let mut pooldata = HashMap::new();
    pooldata.insert(EventAc::Created, cp.pool.created);
    pooldata.insert(EventAc::Deleted, cp.pool.deleted);
    let mut replicadata = HashMap::new();
    replicadata.insert(EventAc::Created, cp.replica.created);
    replicadata.insert(EventAc::Deleted, cp.replica.deleted);
    let mut nexusdata = HashMap::new();
    nexusdata.insert(EventAc::Created, cp.nexus.created);
    nexusdata.insert(EventAc::Deleted, cp.nexus.deleted);
    nexusdata.insert(EventAc::Changed, 0);
    let mut testdata = HashMap::new();
    testdata.insert(EventAc::Created, 0);

    let mut callhomedata = HashMap::new();
    callhomedata.insert(EventCat::Volume, volumedata);
    callhomedata.insert(EventCat::Pool, pooldata);
    callhomedata.insert(EventCat::Replica, replicadata);
    callhomedata.insert(EventCat::Nexus, nexusdata);
    callhomedata.insert(EventCat::Test, testdata);
    UpdatedCallHomeEvent::new("updatedcallhome", UpdatedCallHomeEventSpec {
        events: callhomedata
    })
}