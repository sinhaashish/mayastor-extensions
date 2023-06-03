use std::{collections::BTreeMap, time::Duration, ops::DerefMut};
use k8s_openapi::api::core::v1::ConfigMap;

use anyhow::Result;
use kube::{
    api::{Api, PostParams, PatchParams, Patch},
    Client, core::ObjectMeta,
};
use crate::EventStruct;
use crate::events_cache::Cache;

pub async fn initialize_configmap() -> Result<ConfigMap, kube::Error> {
    let client = Client::try_default().await.unwrap();
    let api: Api<ConfigMap> = Api::namespaced(client.clone(), "mayastor");
    if api.get("events-store-cm").await.is_ok() {
        println!("configmap already exists");
    }
    else {
        println!("creating cm");
        let crd = create_configmap(client.clone()).await;
        println!("cm: {:?}", crd);
    }
    api.get("events-store-cm").await
}

// Create cm
async fn create_configmap(client: Client) -> Result<ConfigMap> {
    println!("creating client");
    let client = Client::try_default().await.unwrap();

    println!("creating api");
    let api: Api<ConfigMap> = Api::namespaced(client.clone(), "mayastor");
    println!("getting data");
    let data = init_config_map();
    println!("init meta for cm");
    let meta = ObjectMeta {
        name: Some("events-store-cm".to_string()),
        ..Default::default()
    };
    println!("init cm data");
    let cm = ConfigMap { data: Some(data), metadata: meta, ..Default::default() };
    // let cm = ConfigMap { data: Some(data), ..Default::default() };
    println!("cm data : {:?}", cm);

    println!("creating cm");
    api.create(&PostParams::default(), &cm).await?;
    println!("Waiting for the api-server to accept the cm");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // It's served by the api - get it and return it
    let cm = api.get("events-store-cm").await?;
    Ok(cm)
}

pub async fn update_configmap() -> Result<(), String> {
    let client = Client::try_default().await.unwrap();
    let api: Api<ConfigMap> = Api::namespaced(client.clone(), "mayastor");
    loop {
        let f1 = update_config_map();
        let meta = ObjectMeta {
            name: Some("events-store-cm".to_string()),
            ..Default::default()
        };
        let cm = ConfigMap { data: Some(f1), metadata: meta, ..Default::default() };
        let ssapply = PatchParams::apply("events_store_configmap").force();
        api.patch("events-store-cm", &ssapply, &Patch::Apply(&cm)).await.unwrap();
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

fn init_config_map() -> BTreeMap<String, String> {
    let cp = EventStruct::default();
    let v = serde_json::to_string(&cp).unwrap();
    let mut data = BTreeMap::new();
    data.insert("stats.json".to_string(), v);
    data

}

fn update_config_map() -> BTreeMap<String, String> {
    let mut c = Cache::get_cache().lock().unwrap();
    let mut binding = c.deref_mut().data_mut();
    let cp = binding.deref_mut();
    let v = serde_json::to_string(&cp).unwrap();
    let mut data = BTreeMap::new();
    data.insert("stats.json".to_string(), v);
    data

}
