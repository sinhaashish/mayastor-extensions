use crate::{
    common::constants::{
        CHART_VERSION_LABEL_KEY, CORE_CHART_NAME, PRODUCT, TO_UMBRELLA_SEMVER, UMBRELLA_CHART_NAME,
        UMBRELLA_CHART_UPGRADE_DOCS_URL,
    },
    upgrade_job::events::event_recorder::EventNote,
};
use snafu::Snafu;
use std::path::PathBuf;
use url::Url;

/// For use with multiple fallible operations which may fail for different reasons, but are
/// defined withing the same scope and must return to the outer scope (calling scope) using
/// the try operator -- '?'.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
#[snafu(context(suffix(false)))]
pub enum Error {
    /// Error for when the storage REST API URL is parsed.
    #[snafu(display(
        "Failed to parse {} REST API URL {}: {}",
        PRODUCT,
        rest_endpoint,
        source
    ))]
    RestUrlParse {
        source: url::ParseError,
        rest_endpoint: String,
    },

    /// Error for when Kubernetes API client generation fails.
    #[snafu(display("Failed to generate kubernetes client: {}", source))]
    K8sClientGeneration { source: kube_client::Error },

    /// Error for a Kubernetes API GET request for a namespace resource fails.
    #[snafu(display("Failed to GET Kubernetes namespace {}: {}", namespace, source))]
    GetNamespace {
        source: kube::Error,
        namespace: String,
    },

    /// Error for when REST API configuration fails.
    #[snafu(display(
        "Failed to configure {} REST API client with endpoint {}: {:?}",
        PRODUCT,
        rest_endpoint,
        source,
    ))]
    RestClientConfiguration {
        #[snafu(source(false))]
        source: openapi::clients::tower::configuration::Error,
        rest_endpoint: Url,
    },

    /// Error for when a Helm command fails.
    #[snafu(display(
        "Failed to run Helm command,\ncommand: {},\nargs: {:?},\ncommand_error: {}",
        command,
        args,
        source
    ))]
    HelmCommand {
        source: std::io::Error,
        command: String,
        args: Vec<String>,
    },

    /// Error for when regular expression parsing or compilation fails.
    #[snafu(display("Failed to compile regex {}: {}", expression, source))]
    RegexCompile {
        source: regex::Error,
        expression: String,
    },

    /// Error for when Helm v3.x.y is not present in $PATH.
    #[snafu(display("Helm version {} does not start with 'v3.x.y'", version))]
    HelmVersion { version: String },

    /// Error for when input Helm release is not found in the input namespace.
    #[snafu(display(
        "'deployed' Helm release {} not found in Namespace {}",
        name,
        namespace
    ))]
    HelmRelease { name: String, namespace: String },

    /// Error for when there is a lack of valid input for the Helm chart directory for the chart to
    /// be upgraded to.
    #[snafu(display("No input for {} helm chart's directory path", chart_name))]
    NoInputHelmChartDir { chart_name: String },

    /// Error for when the input Pod's owner does not exists.
    #[snafu(display(
        ".metadata.ownerReferences empty for Pod {} in {} namespace, while trying to find Pod's Job owner",
        pod_name,
        pod_namespace
    ))]
    JobPodOwnerNotFound {
        pod_name: String,
        pod_namespace: String,
    },

    /// Error for when the number of ownerReferences for this Pod is more than 1.
    #[snafu(display(
        "Pod {} in {} namespace has too many owners, while trying to find Pod's Job owner",
        pod_name,
        pod_namespace
    ))]
    JobPodHasTooManyOwners {
        pod_name: String,
        pod_namespace: String,
    },

    /// Error for when the owner of this Pod is not a Job.
    #[snafu(display(
        "Pod {} in {} namespace has an owner which is not a Job, while trying to find Pod's Job owner",
        pod_name,
        pod_namespace
    ))]
    JobPodOwnerIsNotJob {
        pod_name: String,
        pod_namespace: String,
    },

    /// Error for when yaml could not be parsed from a slice.
    #[snafu(display("Failed to parse YAML {}: {}", input_yaml, source))]
    YamlParseFromSlice {
        source: serde_yaml::Error,
        input_yaml: String,
    },

    /// Error for when yaml could not be parsed from a file (Reader).
    #[snafu(display("Failed to parse YAML at {}: {}", filepath.display(), source))]
    YamlParseFromFile {
        source: serde_yaml::Error,
        filepath: PathBuf,
    },

    /// Error for when yaml could not be parsed from bytes.
    #[snafu(display("Failed to parse unsupported versions yaml: {}", source))]
    YamlParseBufferForUnsupportedVersion { source: serde_yaml::Error },

    /// Error for when the Helm chart installed in the cluster is not of the umbrella or core
    /// variant.
    #[snafu(display(
        "Helm chart release {} in Namespace {} has an unsupported chart variant: {}",
        release_name,
        namespace,
        chart_name
    ))]
    DetermineChartVariant {
        release_name: String,
        namespace: String,
        chart_name: String,
    },

    /// Error for when the path to a directory cannot be validated.
    #[snafu(display("Failed to validate directory path {}: {}", path.display(), source))]
    ValidateDirPath {
        source: std::io::Error,
        path: PathBuf,
    },

    /// Error for when the path to a file cannot be validated.
    #[snafu(display("Failed to validate filepath {}: {}", path.display(), source))]
    ValidateFilePath {
        source: std::io::Error,
        path: PathBuf,
    },

    /// Error for when the path is not that of a directory.
    #[snafu(display("{} is not a directory", path.display()))]
    NotADirectory { path: PathBuf },

    /// Error for when the path is not that of a file.
    #[snafu(display("{} is not a file", path.display()))]
    NotAFile { path: PathBuf },

    /// Error when opening a file.
    #[snafu(display("Failed to open file {}: {}", filepath.display(), source))]
    OpeningFile {
        source: std::io::Error,
        filepath: PathBuf,
    },

    /// Error for when the helm chart found in a path is not of the correct variant.
    #[snafu(display("Failed to find valid Helm chart in path {}", path.display()))]
    FindingHelmChart { path: PathBuf },

    /// Error for when a Kubernetes API request for GET-ing a Pod fails.
    #[snafu(display(
        "Failed to GET Kubernetes Pod {} in namespace {}: {}",
        pod_name,
        pod_namespace,
        source
    ))]
    GetPod {
        source: kube::Error,
        pod_name: String,
        pod_namespace: String,
    },

    /// Error for when a Kubernetes API request for GET-ing a list of Pods filtered by label(s)
    /// fails.
    #[snafu(display(
        "Failed to list Pods with label {} in namespace {}: {}",
        label,
        namespace,
        source
    ))]
    ListPodsWithLabel {
        source: kube::Error,
        label: String,
        namespace: String,
    },

    /// Error for when a Kubernetes API request for GET-ing a list of Pods filtered by label(s)
    /// and field(s) fails.
    #[snafu(display(
        "Failed to list Pods with label '{}', and field '{}' in namespace {}: {}",
        label,
        field,
        namespace,
        source
    ))]
    ListPodsWithLabelAndField {
        source: kube::Error,
        label: String,
        field: String,
        namespace: String,
    },

    /// Error for when a Pod does not have a PodSpec struct member.
    #[snafu(display("Failed get .spec from Pod {} in Namespace {}", name, namespace))]
    EmptyPodSpec { name: String, namespace: String },

    /// Error for when the spec.nodeName of a Pod is empty.
    #[snafu(display(
        "Failed get .spec.nodeName from Pod {} in Namespace {}",
        name,
        namespace
    ))]
    EmptyPodNodeName { name: String, namespace: String },

    /// Error for when the metadata.uid of a Pod is empty.
    #[snafu(display(
        "Failed to get .metadata.uid from Pod {} in Namespace {}",
        name,
        namespace
    ))]
    EmptyPodUid { name: String, namespace: String },

    /// Error for when an uncordon request for a storage node fails.
    #[snafu(display("Failed to uncordon {} Node {}: {}", PRODUCT, node_id, source))]
    StorageNodeUncordon {
        source: openapi::tower::client::Error<openapi::models::RestJsonError>,
        node_id: String,
    },

    /// Error for when an Pod-delete Kubernetes API request fails.
    #[snafu(display("Failed get delete Pod {} from Node {}: {}", name, node, source))]
    PodDelete {
        source: kube::Error,
        name: String,
        node: String,
    },

    /// Error for when listing storage nodes fails.
    #[snafu(display("Failed to list {} Nodes: {}", PRODUCT, source))]
    ListStorageNodes {
        source: openapi::tower::client::Error<openapi::models::RestJsonError>,
    },

    /// Error for when GET-ing a storage node fails.
    #[snafu(display("Failed to list {} Node {}: {}", PRODUCT, node_id, source))]
    GetStorageNode {
        source: openapi::tower::client::Error<openapi::models::RestJsonError>,
        node_id: String,
    },

    /// Error for when the storage node's Spec is empty.
    #[snafu(display("Failed to get {} Node {}", PRODUCT, node_id))]
    EmptyStorageNodeSpec { node_id: String },

    /// Error for when a GET request for a list of storage volumes fails.
    #[snafu(display("Failed to list {} Volumes: {}", PRODUCT, source))]
    ListStorageVolumes {
        source: openapi::tower::client::Error<openapi::models::RestJsonError>,
    },

    /// Error for when a storage node drain request fails.
    #[snafu(display("Failed to drain {} Node {}: {}", PRODUCT, node_id, source))]
    DrainStorageNode {
        source: openapi::tower::client::Error<openapi::models::RestJsonError>,
        node_id: String,
    },

    /// Error for when the requested YAML key is invalid.
    #[snafu(display("Failed to parse YAML path {}", yaml_path))]
    YamlStructure { yaml_path: String },

    /// Error for use when converting Vec<> to String.
    #[snafu(display("Failed to convert Vec<u8> to UTF-8 formatted String: {}", source))]
    U8VectorToString { source: std::str::Utf8Error },

    /// Error when publishing kube-events for the Job object.
    #[snafu(display("Failed to publish Event: {}", source))]
    EventPublish { source: kube_client::Error },

    /// Error for when a Helm list command execution succeeds, but with an error.
    #[snafu(display(
        "`helm list` command return an error,\ncommand: {},\nargs: {:?},\nstd_err: {}",
        command,
        args,
        std_err,
    ))]
    HelmListCommand {
        command: String,
        args: Vec<String>,
        std_err: String,
    },

    /// Error for when a Helm version command execution succeeds, but with an error.
    #[snafu(display(
        "`helm version` command return an error,\ncommand: {},\nargs: {:?},\nstd_err: {}",
        command,
        args,
        std_err,
    ))]
    HelmVersionCommand {
        command: String,
        args: Vec<String>,
        std_err: String,
    },

    /// Error for when a Helm upgrade command execution succeeds, but with an error.
    #[snafu(display(
        "`helm upgrade` command return an error,\ncommand: {},\nargs: {:?},\nstd_err: {}",
        command,
        args,
        std_err,
    ))]
    HelmUpgradeCommand {
        command: String,
        args: Vec<String>,
        std_err: String,
    },

    /// Error for when a Helm get values command execution succeeds, but with an error.
    #[snafu(display(
        "`helm get values` command return an error,\ncommand: {},\nargs: {:?},\nstd_err: {}",
        command,
        args,
        std_err,
    ))]
    HelmGetValuesCommand {
        command: String,
        args: Vec<String>,
        std_err: String,
    },

    /// Error for when detected helm chart name is not known helm chart.
    #[snafu(display(
        "'{}' is not a known {} helm chart, only the '{}' and '{}' charts are supported",
        chart_name,
        PRODUCT,
        CORE_CHART_NAME,
        UMBRELLA_CHART_NAME
    ))]
    NotAKnownHelmChart { chart_name: String },

    /// Error for when namespace option is not set when building KubeClientSet.
    #[snafu(display("Mandatory KubeClientSetBuilder option 'namespace' not set"))]
    KubeClientSetBuilderNs,

    /// Error for when mandatory options for an EventRecorder are missing when building.
    #[snafu(display("Mandatory options for EventRecorder were not given"))]
    EventRecorderOptionsAbsent,

    /// Error for when pod uid is not present.
    #[snafu(display("Pod Uid is None"))]
    PodUidIsNone,

    /// Error for mandatory options for a HelmClient are missing when building.
    #[snafu(display("Setting namespace is mandatory for HelmClient"))]
    HelmClientNs,

    /// Error for mandatory options for a HelmUpgrade are missing when building.
    #[snafu(display("Mandatory options for EventRecorder were not given"))]
    HelmUpgradeOptionsAbsent,

    /// Error for failures in generating semver::Value from a &str input.
    #[snafu(display("Failed to parse {} as a valid semver: {}", version_string, source))]
    SemverParse {
        source: semver::Error,
        version_string: String,
    },

    /// Error for when the detected upgrade path for PRODUCT is not supported.
    #[snafu(display("The upgrade path is invalid"))]
    InvalidUpgradePath,

    /// Error in serializing crate::event::event_recorder::EventNote to JSON string.
    #[snafu(display("Failed to serialize event note {:?}: {}", note, source))]
    SerializeEventNote {
        source: serde_json::Error,
        note: EventNote,
    },

    /// Error for when there are too many io-engine Pods in one single node;
    #[snafu(display("Too many io-engine Pods in Node '{}'", node_name))]
    TooManyIoEnginePods { node_name: String },

    /// Error for when the thin-provisioning options are absent, but still tried to fetch it.
    #[snafu(display("The agents.core.capacity yaml object is absent amongst the helm values"))]
    ThinProvisioningOptionsAbsent,

    /// Error when trying to send Events through the tokio::sync::channel::Sender<Event>
    /// synchronisation tool.
    #[snafu(display("Failed to send Event over the channel"))]
    EventChannelSend,

    /// Error for when the no value for version label is found on the helm chart.
    #[snafu(display(
        "Failed to get the value of the {} label in Pod {} in Namespace {}",
        CHART_VERSION_LABEL_KEY,
        pod_name,
        namespace
    ))]
    HelmChartVersionLabelHasNoValue { pod_name: String, namespace: String },

    /// Error for when a pod does not have Namespace set on it.
    #[snafu(display(
        "Found None when trying to get Namespace for Pod {}, context: {}",
        pod_name,
        context
    ))]
    NoNamespaceInPod { pod_name: String, context: String },

    /// Error for the Umbrella chart is not upgraded.
    #[snafu(display(
        "The {} helm chart is not upgraded to version {}: Upgrade for helm chart {} is not \
        supported, refer to the instructions at {} to upgrade your release of the {} helm \
        chart to version {}",
        UMBRELLA_CHART_NAME,
        TO_UMBRELLA_SEMVER,
        UMBRELLA_CHART_NAME,
        UMBRELLA_CHART_UPGRADE_DOCS_URL,
        UMBRELLA_CHART_NAME,
        TO_UMBRELLA_SEMVER,
    ))]
    UmbrellaChartNotUpgraded,

    /// Error for when the helm upgrade for the Core chart does not have a chart directory.
    #[snafu(display(
        "The {} helm chart could not be upgraded as input chart directory is absent",
        CORE_CHART_NAME
    ))]
    CoreChartUpgradeNoneChartDir,

    /// Error for when the Storage REST API Deployment is absent.
    #[snafu(display(
        "Found no {} REST API Deployments in the namespace {} with labelSelector {}",
        PRODUCT,
        namespace,
        label_selector
    ))]
    NoRestDeployment {
        namespace: String,
        label_selector: String,
    },

    /// Error for when the CHART_VERSION_LABEL_KEY is missing amongst the labels in a Deployment.
    #[snafu(display(
        "A label with the key {} was not found for Deployment {} in namespace {}",
        CHART_VERSION_LABEL_KEY,
        deployment_name,
        namespace
    ))]
    NoVersionLabelInDeployment {
        deployment_name: String,
        namespace: String,
    },

    /// Error for when a Kubernetes API request for GET-ing a list of Deployments filtered by
    /// label(s) fails.
    #[snafu(display(
        "Failed to list Deployments with label {} in namespace {}: {}",
        label_selector,
        namespace,
        source
    ))]
    ListDeploymentsWithLabel {
        source: kube::Error,
        namespace: String,
        label_selector: String,
    },

    /// Error for when the helm upgrade run is that of an invalid chart configuration.
    #[snafu(display("Invalid helm upgrade request"))]
    InvalidHelmUpgrade,

    /// Error for when the helm upgrade run is that of an invalid chart configuration.
    #[snafu(display(
        "Failed to upgrade from {} to {}: upgrade to an earlier-released version is forbidden",
        from_version,
        to_version
    ))]
    RollbackForbidden {
        from_version: String,
        to_version: String,
    },

    /// Error for no upgrade event present.
    #[snafu(display("No upgrade event present."))]
    UpgradeEventNotPresent,

    /// Error for no Deployment present.
    #[snafu(display("No deployment present."))]
    NoDeploymentPresent,

    /// No message in upgrade event.
    #[snafu(display("No Message present in event."))]
    MessageInEventNotPresent,

    /// Nodes are in cordoned state.
    #[snafu(display("Nodes are in cordoned state."))]
    NodesInCordonedState,

    /// Single replica volume present in cluster.
    #[snafu(display("Single replica volume present in cluster."))]
    SingleReplicaVolumeErr,

    /// Cluster is rebuilding replica of some volumes.
    #[snafu(display("Cluster is rebuilding replica of some volumes."))]
    VolumeRebuildInProgress,

    /// Deserialization error for event.
    #[snafu(display("Error in deserializing upgrade event {} Error {}", event, source))]
    EventSerdeDeserialization {
        event: String,
        source: serde_json::Error,
    },

    /// Failed in creating service account.
    #[snafu(display("Service account: {} creation failed Error: {}", name, source))]
    ServiceAccountCreate { name: String, source: kube::Error },

    /// Failed in deletion service account.
    #[snafu(display("Service account: {} deletion failed Error: {}", name, source))]
    ServiceAccountDelete { name: String, source: kube::Error },

    /// Failed in creating cluster role.
    #[snafu(display("Cluster role: {} creation failed Error: {}", name, source))]
    ClusterRoleCreate { name: String, source: kube::Error },

    /// Failed in deletion cluster role.
    #[snafu(display("Cluster role: {} deletion Error: {}", name, source))]
    ClusterRoleDelete { name: String, source: kube::Error },

    /// Failed in deletion cluster role binding.
    #[snafu(display("Cluster role binding: {} deletion failed Error: {}", name, source))]
    ClusterRoleBindingDelete { name: String, source: kube::Error },

    /// Failed in creating cluster role binding.
    #[snafu(display("Cluster role binding: {} creation failed Error: {}", name, source))]
    ClusterRoleBindingCreate { name: String, source: kube::Error },

    /// Failed in creating upgrade job.
    #[snafu(display("Upgrade Job: {} creation failed Error: {}", name, source))]
    UpgradeJobCreate { name: String, source: kube::Error },

    /// Failed in deleting upgrade job.
    #[snafu(display("Upgrade Job: {} deletion failed Error: {}", name, source))]
    UpgradeJobDelete { name: String, source: kube::Error },

    /// Error for when the image format is invalid.
    #[snafu(display("Failed to find a valid image in Deployment."))]
    ReferenceDeploymentInvalidImage,

    /// Error for when the .spec.template.spec.contains[0].image is a None.
    #[snafu(display("Failed to find an image in Deployment."))]
    ReferenceDeploymentNoImage,

    /// Error for when .spec is None for the reference Deployment.
    #[snafu(display("No .spec found for the reference Deployment"))]
    ReferenceDeploymentNoSpec,

    /// Error for when .spec.template.spec is None for the reference Deployment.
    #[snafu(display("No .spec.template.spec found for the reference Deployment"))]
    ReferenceDeploymentNoPodTemplateSpec,

    /// Error for when .spec.template.spec.contains[0] does not exist.
    #[snafu(display("Failed to find the first container of the Deployment."))]
    ReferenceDeploymentNoContainers,

    /// Node spec not present error.
    #[snafu(display("Node spec not present, node: {}", node))]
    NodeSpecNotPresent { node: String },

    /// Error for when the pod.metadata.name is a None.
    #[snafu(display("Pod name not present."))]
    PodNameNotPresent,

    /// Error for when the job.status is a None.
    #[snafu(display("Upgrade Job: {} status not present.", name))]
    UpgradeJobStatusNotPresent { name: String },

    /// Error for when the upgrade job is not present.
    #[snafu(display("Upgrade Job: {} in namespace {} does not exist.", name, namespace))]
    UpgradeJobNotPresent { name: String, namespace: String },

    /// Error for when a Kubernetes API request for GET-ing a list of Deployments filtered by
    /// label(s) fails.
    #[snafu(display(
        "Failed to list Deployments with label {} in namespace {}: {}",
        label,
        namespace,
        source
    ))]
    ListDeploymantsWithLabel {
        source: kube::Error,
        label: String,
        namespace: String,
    },

    /// Error for when a Kubernetes API request for GET-ing a list of events filtered by
    /// filed selector fails.
    #[snafu(display("Failed to list Events with field selector {}: {}", field, source))]
    ListEventsWithFieldSelector { source: kube::Error, field: String },

    /// Error listing the pvc list.
    #[snafu(display("Failed to list pvc : {}", source))]
    ListPVC { source: kube::Error },

    /// Error listing the volumes.
    #[snafu(display("Failed to list volumes : {}", source))]
    ListVolumes {
        source: openapi::tower::client::Error<openapi::models::RestJsonError>,
    },

    /// Error when a Get Upgrade job fails.
    #[snafu(display("Failed to get Upgrade Job {}: {}", name, source))]
    GetUpgradeJob { source: kube::Error, name: String },

    /// Error when a Get ServiceAccount fails.
    #[snafu(display("Failed to get service account {}: {}", name, source))]
    GetServiceAccount { source: kube::Error, name: String },

    /// Error when a Get ClusterRole fails.
    #[snafu(display("Failed to get cluster role {}: {}", name, source))]
    GetClusterRole { source: kube::Error, name: String },

    /// Error when a Get CLusterRoleBinding fails.
    #[snafu(display("Failed to get cluster role binding {}: {}", name, source))]
    GetClusterRoleBinding { source: kube::Error, name: String },

    /// Openapi configuration error.
    #[snafu(display("openapi configuration Error: {}", source))]
    OpenapiClientConfiguration { source: anyhow::Error },

    /// Source and target version are same.
    #[snafu(display("Source and target version are same for upgrade."))]
    SourceTargetVersionSame,

    /// Error when source version is not a valid for upgrade.
    #[snafu(display("Not a valid source version for upgrade."))]
    NotAValidSourceForUpgrade,

    /// K8s client error.
    #[snafu(display("K8Client Error: {}", source))]
    K8sClient { source: kube::Error },
}

/// A wrapper type to remove repeated Result<T, Error> returns.
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<Error> for i32 {
    fn from(err: Error) -> Self {
        match err {
            Error::RestUrlParse { .. } => 401,
            Error::K8sClientGeneration { .. } => 402,
            Error::GetNamespace { .. } => 403,
            Error::RestClientConfiguration { .. } => 404,
            Error::HelmCommand { .. } => 405,
            Error::HelmVersion { .. } => 406,
            Error::HelmRelease { .. } => 407,
            Error::NoInputHelmChartDir { .. } => 408,
            Error::JobPodOwnerNotFound { .. } => 409,
            Error::JobPodHasTooManyOwners { .. } => 410,
            Error::JobPodOwnerIsNotJob { .. } => 411,
            Error::YamlParseFromSlice { .. } => 412,
            Error::YamlParseFromFile { .. } => 413,
            Error::YamlParseBufferForUnsupportedVersion { .. } => 414,
            Error::DetermineChartVariant { .. } => 415,
            Error::ValidateDirPath { .. } => 416,
            Error::ValidateFilePath { .. } => 417,
            Error::NotADirectory { .. } => 418,
            Error::NotAFile { .. } => 419,
            Error::OpeningFile { .. } => 420,
            Error::FindingHelmChart { .. } => 421,
            Error::GetPod { .. } => 422,
            Error::ListPodsWithLabel { .. } => 423,
            Error::ListPodsWithLabelAndField { .. } => 424,
            Error::EmptyPodSpec { .. } => 425,
            Error::EmptyPodNodeName { .. } => 426,
            Error::EmptyPodUid { .. } => 427,
            Error::StorageNodeUncordon { .. } => 428,
            Error::PodDelete { .. } => 429,
            Error::ListStorageNodes { .. } => 430,
            Error::GetStorageNode { .. } => 431,
            Error::EmptyStorageNodeSpec { .. } => 432,
            Error::ListStorageVolumes { .. } => 433,
            Error::DrainStorageNode { .. } => 434,
            Error::YamlStructure { .. } => 435,
            Error::U8VectorToString { .. } => 436,
            Error::EventPublish { .. } => 437,
            Error::HelmListCommand { .. } => 438,
            Error::HelmVersionCommand { .. } => 439,
            Error::HelmUpgradeCommand { .. } => 440,
            Error::HelmGetValuesCommand { .. } => 441,
            Error::NotAKnownHelmChart { .. } => 442,
            Error::KubeClientSetBuilderNs { .. } => 443,
            Error::EventRecorderOptionsAbsent { .. } => 444,
            Error::PodUidIsNone { .. } => 445,
            Error::HelmClientNs { .. } => 446,
            Error::HelmUpgradeOptionsAbsent { .. } => 446,
            Error::SemverParse { .. } => 447,
            Error::InvalidUpgradePath { .. } => 448,
            Error::SerializeEventNote { .. } => 449,
            Error::TooManyIoEnginePods { .. } => 450,
            Error::ThinProvisioningOptionsAbsent { .. } => 451,
            Error::EventChannelSend { .. } => 452,
            Error::HelmChartVersionLabelHasNoValue { .. } => 453,
            Error::NoNamespaceInPod { .. } => 454,
            Error::UmbrellaChartNotUpgraded { .. } => 455,
            Error::CoreChartUpgradeNoneChartDir { .. } => 456,
            Error::NoRestDeployment { .. } => 457,
            Error::NoVersionLabelInDeployment { .. } => 458,
            Error::ListDeploymentsWithLabel { .. } => 459,
            Error::InvalidHelmUpgrade { .. } => 460,
            Error::RollbackForbidden { .. } => 461,
            Error::UpgradeEventNotPresent { .. } => 462,
            Error::NoDeploymentPresent { .. } => 463,
            Error::MessageInEventNotPresent { .. } => 464,
            Error::NodesInCordonedState { .. } => 465,
            Error::SingleReplicaVolumeErr { .. } => 466,
            Error::VolumeRebuildInProgress { .. } => 467,
            Error::EventSerdeDeserialization { .. } => 468,
            Error::ServiceAccountCreate { .. } => 469,
            Error::ServiceAccountDelete { .. } => 470,
            Error::ClusterRoleCreate { .. } => 471,
            Error::ClusterRoleDelete { .. } => 472,
            Error::ClusterRoleBindingDelete { .. } => 473,
            Error::ClusterRoleBindingCreate { .. } => 474,
            Error::UpgradeJobCreate { .. } => 475,
            Error::UpgradeJobDelete { .. } => 476,
            Error::ReferenceDeploymentInvalidImage { .. } => 477,
            Error::ReferenceDeploymentNoImage { .. } => 478,
            Error::ReferenceDeploymentNoSpec { .. } => 479,
            Error::ReferenceDeploymentNoPodTemplateSpec { .. } => 480,
            Error::ReferenceDeploymentNoContainers { .. } => 481,
            Error::NodeSpecNotPresent { .. } => 482,
            Error::PodNameNotPresent { .. } => 483,
            Error::UpgradeJobStatusNotPresent { .. } => 484,
            Error::UpgradeJobNotPresent { .. } => 485,
            Error::ListDeploymantsWithLabel { .. } => 486,
            Error::ListEventsWithFieldSelector { .. } => 487,
            Error::ListPVC { .. } => 488,
            Error::ListVolumes { .. } => 489,
            Error::GetUpgradeJob { .. } => 490,
            Error::GetServiceAccount { .. } => 491,
            Error::GetClusterRole { .. } => 492,
            Error::GetClusterRoleBinding { .. } => 493,
            Error::OpenapiClientConfiguration { .. } => 494,
            Error::SourceTargetVersionSame { .. } => 495,
            Error::NotAValidSourceForUpgrade { .. } => 496,
            Error::K8sClient { .. } => 497,
            Error::RegexCompile { .. } => 498,
        }
    }
}
