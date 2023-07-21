use k8s_openapi::api::core::v1::Pod;
use kube::{api::ObjectList, ResourceExt};
use semver::{Version, VersionReq};
use snafu::ResultExt;
use tracing::{info, warn};
use upgrade::common::{
    constants::CHART_VERSION_LABEL_KEY,
    error::{HelmChartVersionLabelHasNoValue, NoNamespaceInPod, Result, SemverParse},
};

/// This function returns 'true' only if all of the containers in the Pods contained in the
/// ObjectList<Pod> have their Ready status.condition value set to true.
pub(crate) fn all_pods_are_ready(pod_list: ObjectList<Pod>) -> bool {
    let not_ready_warning = |pod_name: &String, namespace: &String| {
        warn!(
            "Couldn't verify the ready condition of Pod '{}' in namespace '{}' to be true",
            pod_name, namespace
        );
    };
    for pod in pod_list.into_iter() {
        match &pod
            .status
            .as_ref()
            .and_then(|status| status.conditions.as_ref())
        {
            Some(conditions) => {
                for condition in *conditions {
                    if condition.type_.eq("Ready") {
                        if condition.status.eq("True") {
                            let pod_name = pod.name_any();
                            info!(pod.name = %pod_name, "Pod is Ready");
                            break;
                        }
                        not_ready_warning(&pod.name_any(), &pod.namespace().unwrap_or_default());
                        return false;
                    } else {
                        continue;
                    }
                }
            }
            None => {
                not_ready_warning(&pod.name_any(), &pod.namespace().unwrap_or_default());
                return false;
            }
        }
    }
    true
}

/// Checks to see if all of io-engine Pods are already upgraded to the version of the local helm
/// chart.
pub(crate) async fn data_plane_is_upgraded(
    to_version: &str,
    io_engine_pod_list: &ObjectList<Pod>,
) -> Result<bool> {
    let to_version_requirement: VersionReq =
        VersionReq::parse(to_version).context(SemverParse {
            version_string: to_version.to_string(),
        })?;

    for pod in io_engine_pod_list {
        let version_str = pod.labels().get(CHART_VERSION_LABEL_KEY).ok_or(
            HelmChartVersionLabelHasNoValue {
                pod_name: pod.name_any(),
                namespace: pod.namespace().ok_or(
                    NoNamespaceInPod {
                        pod_name: pod.name_any(),
                        context: "checking to see if data-plane Pods are already upgraded"
                            .to_string(),
                    }
                    .build(),
                )?,
            }
            .build(),
        )?;
        let version = Version::parse(version_str).context(SemverParse {
            version_string: version_str.clone(),
        })?;
        if !to_version_requirement.matches(&version) {
            return Ok(false);
        }
    }

    Ok(true)
}
