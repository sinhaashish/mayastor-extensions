use crate::common::{
    constants::{
        API_CALL_HOME_LABEL_SELECTOR, API_REST_LABEL_SELECTOR, DEFAULT_RELEASE_NAME,
        HELM_RELEASE_NAME_LABEL,
    },
    errors,
};
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Service};
use kube::{
    api::{Api, ListParams},
    Client,
};
use snafu::ResultExt;

/// Return the release name.
pub async fn release_name(ns: &str, client: Client) -> errors::Result<String> {
    let deployment = deployment_for_rest(ns, client).await?;
    match &deployment.metadata.labels {
        Some(label) => match label.get(HELM_RELEASE_NAME_LABEL) {
            Some(value) => Ok(value.to_string()),
            None => Ok(DEFAULT_RELEASE_NAME.to_string()),
        },
        None => Ok(DEFAULT_RELEASE_NAME.to_string()),
    }
}

/// Return results as list of deployments.
pub async fn deployment_for_rest(ns: &str, client: Client) -> errors::Result<Deployment> {
    let deployment = Api::<Deployment>::namespaced(client, ns);
    let lp = ListParams::default().labels(API_REST_LABEL_SELECTOR);
    let deployment_list = deployment
        .list(&lp)
        .await
        .context(errors::ListDeploymentsWithLabel {
            label: API_REST_LABEL_SELECTOR.to_string(),
            namespace: ns.to_string(),
        })?;
    let deployment = deployment_list
        .items
        .first()
        .ok_or(errors::NoDeploymentPresent.build())?
        .clone();
    Ok(deployment)
}

/// Fetch the events stats.
pub async fn event_stats(ns: &str) -> errors::Result<()> {
    tracing::info!("Ashish kumar");
    //let release_name = release_name(namespace, client.clone()).await?;
    let client = Client::try_default().await.context(errors::K8sClient)?;
    let service = Api::<Service>::namespaced(client, ns);
    let lp = ListParams::default().labels(API_CALL_HOME_LABEL_SELECTOR);
    let services = service
        .list(&lp)
        .await
        .context(errors::ListServiceWithLabel {
            label: API_CALL_HOME_LABEL_SELECTOR.to_string(),
            namespace: ns.to_string(),
        })?;

    let svc = services
        .items
        .first()
        .ok_or(errors::ServiceNameNotPresent.build())?;

    let service_name = svc
        .metadata
        .name
        .clone()
        .ok_or(errors::ServiceNameNotPresent.build())?;

    let service_port = svc
        .spec
        .clone()
        .ok_or(errors::ReferenceServiceNoSpec.build())?
        .ports
        .ok_or(errors::NoPortsPresent.build())?
        .first()
        .ok_or(errors::NoPortsPresent.build())?
        .port;

    let url = format!("http://{}:{}/stats", service_name, service_port);
    tracing::info!(url);

    let request = reqwest::Client::new().get(url);
    //.context(errors::GetEventsStats)?;

    // let request = reqwest::Client::new()
    //     .get(url)
    //     .timeout(std::time::Duration::from_millis(500));

    let res = request.send().await.context(errors::GetEventsStats)?;

    eprintln!("Response: {:?} {}", res.version(), res.status());
    eprintln!("Headers: {:#?}\n", res.headers());

    let body = res.text().await.context(errors::GetEventsStats)?;

    println!("{}", body);
    Ok(())
    // match request.send().await {
    //     Ok(resp) if resp.status().is_success() => {
    //         break;
    //     }
    //     _ => {}
    // }

    //     let request = reqwest::Client::new()

    //     let mut pools_size_vector = Vec::with_capacity(pools.len());
    //     for pool in pools.iter() {
    //         match &pool.state {
    //             Some(pool_state) => pools_size_vector.push(pool_state.capacity),
    //             None => {}
    //         };
    //     }
    //     pools_size_vector
}
