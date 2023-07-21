use crate::common::error;
use openapi::clients::tower::{self, Configuration};
use snafu::ResultExt;
use std::ops::Deref;

use crate::common::error::{RestClientConfiguration, RestUrlParse};
use openapi::tower::client::{ApiClient, Configuration as RestConfig};
use std::time::Duration;
use url::Url;

/// Function to check for any volume rebuild in progress across the cluster
pub async fn is_rebuilding(rest_client: &RestClientSet) -> error::Result<bool> {
    // The number of volumes to get per request.
    let max_entries = 200;
    let mut starting_token = Some(0_isize);

    // The last paginated request will set the `starting_token` to `None`.
    while starting_token.is_some() {
        let vols = rest_client
            .volumes_api()
            .get_volumes(max_entries, None, starting_token)
            .await
            .context(error::ListVolumes)?;

        let volumes = vols.into_body();
        starting_token = volumes.next_token;
        for volume in volumes.entries {
            if let Some(target) = &volume.state.target {
                if target
                    .children
                    .iter()
                    .any(|child| child.rebuild_progress.is_some())
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

// /// New-Type for a RestClient over the tower openapi client.
// #[derive(Clone, Debug)]
// pub struct RestClient {
//     client: tower::ApiClient,
// }

// impl Deref for RestClient {
//     type Target = tower::ApiClient;

//     fn deref(&self) -> &Self::Target {
//         &self.client
//     }
// }

// impl RestClient {

// }

/// This is a wrapper for the openapi::tower::client::ApiClient.
#[derive(Clone, Debug)]
pub struct RestClientSet {
    client: ApiClient,
}

impl Deref for RestClientSet {
    type Target = tower::ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl RestClientSet {
    /// Create new Rest Client from the given `Configuration`.
    pub fn new_with_config(config: Configuration) -> RestClientSet {
        Self {
            client: tower::ApiClient::new(config),
        }
    }

    /// Build the RestConfig, and the eventually the ApiClient. Fails if configuration is invalid.
    pub fn new_with_url(rest_endpoint: String) -> error::Result<Self> {
        let rest_url =
            Url::try_from(rest_endpoint.as_str()).context(RestUrlParse { rest_endpoint })?;

        let config = RestConfig::builder()
            .with_timeout(Duration::from_secs(30))
            .with_tracing(true)
            .build_url(rest_url.clone())
            .map_err(|e| {
                RestClientConfiguration {
                    source: e,
                    rest_endpoint: rest_url,
                }
                .build()
            })?;
        let client = ApiClient::new(config);

        Ok(RestClientSet { client })
    }

    pub(crate) fn nodes_api(&self) -> &dyn openapi::apis::nodes_api::tower::client::Nodes {
        self.client.nodes_api()
    }

    pub(crate) fn volumes_api(&self) -> &dyn openapi::apis::volumes_api::tower::client::Volumes {
        self.client.volumes_api()
    }
}

// Check for rebuild in progress.
// pub(crate) async fn is_rebuild_in_progress(client: &RestClient) -> error::Result<bool> {
//     // The number of volumes to get per request.
//     let max_entries = 200;
//     let mut starting_token = Some(0_isize);

//     // The last paginated request will set the `starting_token` to `None`.
//     while starting_token.is_some() {
//         let vols = client
//             .volumes_api()
//             .get_volumes(max_entries, None, starting_token)
//             .await
//             .context(error::ListVolumes)?;
//         let volumes = vols.into_body();
//         starting_token = volumes.next_token;
//         for volume in volumes.entries {
//             if let Some(target) = &volume.state.target {
//                 if target
//                     .children
//                     .iter()
//                     .any(|child| child.rebuild_progress.is_some())
//                 {
//                     return Ok(true);
//                 }
//             }
//         }
//     }
//     Ok(false)
// }
