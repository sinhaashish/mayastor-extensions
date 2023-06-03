use std::{env, fmt::Debug, ops::DerefMut};

use prometheus::{
    core::{Collector, Desc},
    GaugeVec, Opts,
};
use tracing::error;

use crate::events_cache::{Cache, EventStruct};

/// StatsCollector contains the list of custom metrics that has to be exposed by exporter.
#[derive(Clone, Debug)]
pub struct StatsCollector {
    volumes: GaugeVec,
    nexuses: GaugeVec,
    pools: GaugeVec,
    replicas: GaugeVec,
    descs: Vec<Desc>,
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsCollector {
    /// Initialize all the metrics to be defined for stats collector.
    pub fn new() -> Self {
        let volume_opts = Opts::new("volume", "Volume stat").variable_labels(vec!["action".to_string()]);
        let nexus_opts = Opts::new("nexus", "Nexus stat").variable_labels(vec!["action".to_string()]);
        let pool_opts = Opts::new("pool", "Pool stat").variable_labels(vec!["action".to_string()]);
        let replica_opts = Opts::new("replica", "Replica stat").variable_labels(vec!["action".to_string()]);
        let mut descs = Vec::new();

        let volumes = GaugeVec::new(volume_opts, &["action"])
            .expect("Unable to create gauge metric type for volume stats");
        let nexuses = GaugeVec::new(nexus_opts, &["action"])
            .expect("Unable to create gauge metric type for nexus stats");
        let pools = GaugeVec::new(pool_opts, &["action"])
            .expect("Unable to create gauge metric type for pool stats");
        let replicas = GaugeVec::new(replica_opts, &["action"])
            .expect("Unable to create gauge metric type for replica stats");
        descs.extend(volumes.desc().into_iter().cloned());
        descs.extend(nexuses.desc().into_iter().cloned());
        descs.extend(pools.desc().into_iter().cloned());
        descs.extend(replicas.desc().into_iter().cloned());

        Self {
            volumes,
            nexuses,
            pools,
            replicas,
            descs,
        }
    }

    fn get_volume_metrics(&self, events: &mut EventStruct) -> Vec<prometheus::proto::MetricFamily> {

        let mut metric_family = Vec::new();
        let volumes_created = match self.volumes.get_metric_with_label_values(&["created"])
        {
            Ok(volumes) => volumes,
            Err(error) => {
                error!(%error,"Error while creating metrics(volumes created) with label values:");
                return metric_family;
            }
        };
        volumes_created.set(events.volume.created as f64);
        println!("{}", events.volume.created);
        let volumes_deleted = match self.volumes.get_metric_with_label_values(&["deleted"])
        {
            Ok(volumes) => volumes,
            Err(error) => {
                error!(%error,"Error while creating metrics(volumes deleted) with label values:");
                return metric_family;
            }
        };
        volumes_deleted.set(events.volume.deleted as f64);
        metric_family.extend(volumes_created.collect().pop());
        metric_family.extend(volumes_deleted.collect().pop());
        println!("{metric_family:?}");
        metric_family
    }

    fn get_nexus_metrics(&self, events: &mut EventStruct) -> Vec<prometheus::proto::MetricFamily> {

        let mut metric_family = Vec::new();
        let nexuses_created = match self.nexuses.get_metric_with_label_values(&["created"])
        {
            Ok(nexuses) => nexuses,
            Err(error) => {
                error!(%error,"Error while creating metrics(nexuses created) with label values:");
                return metric_family;
            }
        };
        nexuses_created.set(events.nexus.created as f64);
        let nexuses_deleted = match self.nexuses.get_metric_with_label_values(&["deleted"])
        {
            Ok(nexuses) => nexuses,
            Err(error) => {
                error!(%error,"Error while creating metrics(nexuses deleted) with label values:");
                return metric_family;
            }
        };
        nexuses_deleted.set(events.nexus.deleted as f64);
        metric_family.extend(nexuses_created.collect().pop());
        metric_family.extend(nexuses_deleted.collect().pop());
        metric_family
    }

    fn get_pool_metrics(&self, events: &mut EventStruct) -> Vec<prometheus::proto::MetricFamily> {

        let mut metric_family = Vec::new();
        let pools_created = match self.pools.get_metric_with_label_values(&["created"])
        {
            Ok(pools) => pools,
            Err(error) => {
                error!(%error,"Error while creating metrics(pools created) with label values:");
                return metric_family;
            }
        };
        pools_created.set(events.pool.created as f64);
        println!("{}", events.pool.created);
        let pools_deleted = match self.pools.get_metric_with_label_values(&["deleted"])
        {
            Ok(pools) => pools,
            Err(error) => {
                error!(%error,"Error while creating metrics(pools deleted) with label values:");
                return metric_family;
            }
        };
        pools_deleted.set(events.pool.deleted as f64);
        metric_family.extend(pools_created.collect().pop());
        metric_family.extend(pools_deleted.collect().pop());
        println!("{metric_family:?}");
        metric_family
    }

    fn get_replica_metrics(&self, events: &mut EventStruct) -> Vec<prometheus::proto::MetricFamily> {

        let mut metric_family = Vec::new();
        let replicas_created = match self.replicas.get_metric_with_label_values(&["created"])
        {
            Ok(replicas) => replicas,
            Err(error) => {
                error!(%error,"Error while creating metrics(replicas created) with label values:");
                return metric_family;
            }
        };
        replicas_created.set(events.replica.created as f64);
        println!("{}", events.replica.created);
        let replicas_deleted = match self.replicas.get_metric_with_label_values(&["deleted"])
        {
            Ok(replicas) => replicas,
            Err(error) => {
                error!(%error,"Error while creating metrics(replicas deleted) with label values:");
                return metric_family;
            }
        };
        replicas_deleted.set(events.replica.deleted as f64);
        metric_family.extend(replicas_created.collect().pop());
        metric_family.extend(replicas_deleted.collect().pop());
        println!("{metric_family:?}");
        metric_family
    }
}

/// Prometheus collector implementation
impl Collector for StatsCollector {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        self.descs.iter().collect()
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        let mut c = match Cache::get_cache().lock() {
            Ok(c) => c,
            Err(error) => {
                error!(%error,"Error while getting stats cache resource");
                return Vec::new();
            }
        };
        let cp = c.deref_mut();
        let mut metric_family = Vec::new();
        metric_family.extend(self.get_volume_metrics(cp.data_mut().deref_mut()));
        metric_family.extend(self.get_nexus_metrics(cp.data_mut().deref_mut()));
        metric_family.extend(self.get_pool_metrics(cp.data_mut().deref_mut()));
        metric_family.extend(self.get_replica_metrics(cp.data_mut().deref_mut()));
        metric_family
    }

}
