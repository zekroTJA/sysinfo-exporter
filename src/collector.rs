use anyhow::Result;
use prometheus::{
    Gauge, Opts, Registry,
    core::{AtomicU64, GenericCounter, GenericGauge},
    proto::MetricFamily,
};
use std::collections::HashMap;
use sysinfo::{Disks, Networks, System};

struct DiskMetrics {
    free_gauge: GenericGauge<AtomicU64>,
    total_gauge: GenericGauge<AtomicU64>,
}

struct NetworkMetrics {
    received_counter: GenericCounter<AtomicU64>,
    transmitted_counter: GenericCounter<AtomicU64>,
}

pub struct Collector {
    registry: Registry,

    system: System,
    disks: Disks,
    networks: Networks,

    cpu_usage: Gauge,
    used_memory: GenericGauge<AtomicU64>,
    total_memory: GenericGauge<AtomicU64>,

    disk_metrics: HashMap<String, DiskMetrics>,
    network_metrics: HashMap<String, NetworkMetrics>,
}

impl Collector {
    pub fn new() -> Result<Self> {
        let system = System::new_all();
        let mut disks = Disks::new();
        let mut networks = Networks::new();

        let registry = Registry::new();

        let cpu_usage = Gauge::with_opts(Opts::new("sysinfo_cpu_usage", "Average CPU usage"))?;
        let used_memory =
            GenericGauge::with_opts(Opts::new("sysinfo_memory_used", "Used memory in bytes"))?;
        let total_memory =
            GenericGauge::with_opts(Opts::new("sysinfo_memory_total", "Total memory in bytes"))?;

        registry.register(Box::new(cpu_usage.clone()))?;
        registry.register(Box::new(used_memory.clone()))?;
        registry.register(Box::new(total_memory.clone()))?;

        let mut disk_metrics = HashMap::new();
        disks.refresh(true);
        for disk in &disks {
            let free_gauge = GenericGauge::with_opts(
                Opts::new("sysinfo_disk_free", "Drive free space in bytes")
                    .const_label("name", disk.name().to_string_lossy())
                    .const_label("mount", disk.mount_point().to_string_lossy()),
            )?;
            let total_gauge = GenericGauge::with_opts(
                Opts::new("sysinfo_disk_total", "Drive total space in bytes")
                    .const_label("name", disk.name().to_string_lossy())
                    .const_label("mount", disk.mount_point().to_string_lossy()),
            )?;
            registry.register(Box::new(free_gauge.clone()))?;
            registry.register(Box::new(total_gauge.clone()))?;
            disk_metrics.insert(
                disk.mount_point().to_string_lossy().to_string(),
                DiskMetrics {
                    free_gauge,
                    total_gauge,
                },
            );
        }

        let mut network_metrics = HashMap::new();
        networks.refresh(true);
        for (name, _) in &networks {
            let received_counter = GenericCounter::with_opts(
                Opts::new(
                    "sysinfo_network_total_received",
                    "Total received network bytes",
                )
                .const_label("name", name),
            )?;
            let transmitted_counter = GenericCounter::with_opts(
                Opts::new(
                    "sysinfo_network_total_transmitted",
                    "Total transmitted network bytes",
                )
                .const_label("name", name),
            )?;
            registry.register(Box::new(received_counter.clone()))?;
            registry.register(Box::new(transmitted_counter.clone()))?;
            network_metrics.insert(name.to_owned(), NetworkMetrics {
                received_counter,
                transmitted_counter,
            });
        }

        Ok(Self {
            disks,
            system,
            networks,
            registry,
            cpu_usage,
            used_memory,
            total_memory,
            disk_metrics,
            network_metrics,
        })
    }

    pub fn collect(&mut self) {
        self.system.refresh_all();

        self.cpu_usage.set(self.system.global_cpu_usage() as f64);
        self.used_memory.set(self.system.used_memory());
        self.total_memory.set(self.system.total_memory());

        self.disks.refresh(false);
        for disk in &self.disks {
            let Some(mount) = disk.mount_point().to_str() else {
                continue;
            };
            let Some(metrics) = self.disk_metrics.get(mount) else {
                continue;
            };
            metrics.free_gauge.set(disk.available_space());
            metrics.total_gauge.set(disk.total_space());
        }

        self.networks.refresh(false);
        for (name, network) in &self.networks {
            let Some(metrics) = self.network_metrics.get(name) else {
                continue;
            };
            metrics.received_counter.inc_by(network.received());
            metrics.transmitted_counter.inc_by(network.transmitted());
        }
    }

    pub fn gather(&self) -> Vec<MetricFamily> {
        self.registry.gather()
    }
}
