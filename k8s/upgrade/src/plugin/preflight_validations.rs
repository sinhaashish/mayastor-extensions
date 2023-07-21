use crate::{
    common::{
        constants::{get_image_version_tag, SINGLE_REPLICA_VOLUME, UPGRADE_TO_DEVELOP_BRANCH},
        error::{
            InvalidUpgradePath, ListStorageNodes, ListVolumes, NodeSpecNotPresent,
            NodesInCordonedState, NotAValidSourceForUpgrade, OpenapiClientConfiguration, Result,
            SemverParse, SingleReplicaVolumeErr, VolumeRebuildInProgress,
            YamlParseBufferForUnsupportedVersion,
        },
        utils::{is_rebuilding, RestClientSet},
    },
    plugin::{
        upgrade::{get_pvc_from_uuid, get_source_version},
        user_prompt,
    },
};
use openapi::models::CordonDrainState;
use semver::Version;
use serde::Deserialize;
use serde_yaml;
use snafu::ResultExt;
use std::{collections::HashSet, path::PathBuf};

/// Validation to be done before applying upgrade.
pub async fn preflight_check(
    namespace: &str,
    kube_config_path: Option<PathBuf>,
    timeout: humantime::Duration,
    skip_single_replica_volume_validation: bool,
    skip_replica_rebuild: bool,
    skip_cordoned_node_validation: bool,
    skip_upgrade_path_validation: bool,
) -> Result<()> {
    console_logger::info(user_prompt::UPGRADE_WARNING, "");
    // Initialise the REST client.
    let config = kube_proxy::ConfigBuilder::default_api_rest()
        .with_kube_config(kube_config_path.clone())
        .with_timeout(*timeout)
        .with_target_mod(|t| t.with_namespace(namespace))
        .build()
        .await
        .context(OpenapiClientConfiguration)?;
    let rest_client = RestClientSet::new_with_config(config);

    if !skip_upgrade_path_validation {
        upgrade_path_validation(namespace).await?;
    }

    if !skip_replica_rebuild {
        rebuild_in_progress_validation(&rest_client).await?;
    }

    if !skip_cordoned_node_validation {
        already_cordoned_nodes_validation(&rest_client).await?;
    }

    if !skip_single_replica_volume_validation {
        single_volume_replica_validation(&rest_client).await?;
    }
    Ok(())
}

/// Prompt to user and error out if some nodes are already in cordoned state.
pub(crate) async fn already_cordoned_nodes_validation(client: &RestClientSet) -> Result<()> {
    let mut cordoned_nodes_list = Vec::new();
    let nodes = client
        .nodes_api()
        .get_nodes(None)
        .await
        .context(ListStorageNodes)?;
    let nodelist = nodes.into_body();
    for node in nodelist {
        let node_spec = node.spec.ok_or(
            NodeSpecNotPresent {
                node: node.id.to_string(),
            }
            .build(),
        )?;

        if matches!(
            node_spec.cordondrainstate,
            Some(CordonDrainState::cordonedstate(_))
        ) {
            cordoned_nodes_list.push(node.id);
        }
    }
    if !cordoned_nodes_list.is_empty() {
        console_logger::error(
            user_prompt::CORDONED_NODE_WARNING,
            &cordoned_nodes_list.join("\n"),
        );
        return NodesInCordonedState.fail();
    }
    Ok(())
}

/// Prompt to user and error out if the cluster has single replica volume.
pub(crate) async fn single_volume_replica_validation(client: &RestClientSet) -> Result<()> {
    // let mut single_replica_volumes = Vec::new();
    // The number of volumes to get per request.
    let max_entries = 200;
    let mut starting_token = Some(0_isize);
    let mut volumes = Vec::with_capacity(max_entries as usize);

    // The last paginated request will set the `starting_token` to `None`.
    while starting_token.is_some() {
        let vols = client
            .volumes_api()
            .get_volumes(max_entries, None, starting_token)
            .await
            .context(ListVolumes)?;

        let v = vols.into_body();
        let single_rep_vol_ids: Vec<String> = v
            .entries
            .into_iter()
            .filter(|volume| volume.spec.num_replicas == SINGLE_REPLICA_VOLUME)
            .map(|volume| volume.spec.uuid.to_string())
            .collect();
        volumes.extend(single_rep_vol_ids);
        starting_token = v.next_token;
    }

    if !volumes.is_empty() {
        let data = get_pvc_from_uuid(HashSet::from_iter(volumes))
            .await?
            .join("\n");

        console_logger::error(user_prompt::SINGLE_REPLICA_VOLUME_WARNING, &data);
        return SingleReplicaVolumeErr.fail();
    }
    Ok(())
}

/// Prompt to user and error out if any rebuild in progress.
pub(crate) async fn rebuild_in_progress_validation(client: &RestClientSet) -> Result<()> {
    if is_rebuilding(client).await? {
        console_logger::error(user_prompt::REBUILD_WARNING, "");
        return VolumeRebuildInProgress.fail();
    }
    Ok(())
}

/// Struct to deserialize the unsupported version yaml.
#[derive(Deserialize)]
struct UnsupportedVersions {
    unsupported_versions: Vec<Version>,
}

impl UnsupportedVersions {
    fn contains(&self, v: &Version) -> bool {
        self.unsupported_versions.contains(v)
    }
}

impl TryFrom<&[u8]> for UnsupportedVersions {
    type Error = serde_yaml::Error;

    /// Returns an UnsupportedVersions object.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        serde_yaml::from_reader(bytes)
    }
}

pub(crate) async fn upgrade_path_validation(namespace: &str) -> Result<()> {
    let unsupported_version_buf =
        &std::include_bytes!("../../config/unsupported_versions.yaml")[..];
    let unsupported_versions = UnsupportedVersions::try_from(unsupported_version_buf)
        .context(YamlParseBufferForUnsupportedVersion)?;
    let source_version = get_source_version(namespace).await?;

    let source = Version::parse(source_version.as_str()).context(SemverParse {
        version_string: source_version.clone(),
    })?;

    if unsupported_versions.contains(&source) {
        let mut invalid_source_list: String = Default::default();
        for val in unsupported_versions.unsupported_versions.iter() {
            invalid_source_list.push_str(val.to_string().as_str());
            invalid_source_list.push('\n');
        }
        console_logger::error(
            user_prompt::UPGRADE_PATH_NOT_VALID,
            invalid_source_list.as_str(),
        );
        return NotAValidSourceForUpgrade.fail();
    }
    let destination_version = get_image_version_tag();

    if destination_version.contains(UPGRADE_TO_DEVELOP_BRANCH) {
        console_logger::error("", user_prompt::UPGRADE_TO_UNSUPPORTED_VERSION);
        return InvalidUpgradePath.fail();
    }
    Ok(())
}
