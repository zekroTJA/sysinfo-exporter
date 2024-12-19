use anyhow::Result;
use prometheus::{
    Gauge, Opts, Registry,
    core::{AtomicU64, GenericGauge},
    proto::MetricFamily,
};
use std::collections::HashMap;
use sysinfo::{Disks, System};

pub struct Collector {
    registry: Registry,

    system: System,
    disks: Disks,

    cpu_usage: Gauge,
    used_memory: GenericGauge<AtomicU64>,
    available_memory: GenericGauge<AtomicU64>,

    disk_gauges: HashMap<String, (GenericGauge<AtomicU64>, GenericGauge<AtomicU64>)>,
}

impl Collector {
    pub fn new() -> Result<Self> {
        let system = System::new_all();
        let mut disks = Disks::new();
        let registry = Registry::new();

        let cpu_usage = Gauge::with_opts(Opts::new("sysinfo_cpu_usage", "Average CPU usage"))?;
        let used_memory =
            GenericGauge::with_opts(Opts::new("sysinfo_used_memory", "Used memory in bytes"))?;
        let available_memory = GenericGauge::with_opts(Opts::new(
            "sysinfo_available_memory",
            "Available memory in bytes",
        ))?;

        registry.register(Box::new(cpu_usage.clone()))?;
        registry.register(Box::new(used_memory.clone()))?;
        registry.register(Box::new(available_memory.clone()))?;

        let mut disk_gauges = HashMap::new();
        disks.refresh(true);
        for disk in &disks {
            let free_guage = GenericGauge::with_opts(
                Opts::new("sysinfo_disk_free", "Drive free space in bytes")
                    .const_label("name", disk.name().to_string_lossy())
                    .const_label("mount", disk.mount_point().to_string_lossy()),
            )?;
            let total_gauge = GenericGauge::with_opts(
                Opts::new("sysinfo_disk_total", "Drive total space in bytes")
                    .const_label("name", disk.name().to_string_lossy())
                    .const_label("mount", disk.mount_point().to_string_lossy()),
            )?;
            registry.register(Box::new(free_guage.clone()))?;
            registry.register(Box::new(total_gauge.clone()))?;
            disk_gauges.insert(
                disk.mount_point().to_string_lossy().to_string(),
                (free_guage, total_gauge),
            );
        }

        Ok(Self {
            disks,
            system,
            registry,
            cpu_usage,
            used_memory,
            available_memory,
            disk_gauges,
        })
    }

    pub fn collect(&mut self) {
        self.system.refresh_all();

        self.cpu_usage.set(self.system.global_cpu_usage() as f64);
        self.used_memory.set(self.system.used_memory());
        self.available_memory.set(self.system.available_memory());

        self.disks.refresh(false);

        for disk in &self.disks {
            let Some(mount) = disk.mount_point().to_str() else {
                continue;
            };
            let Some((free_gauge, total_gauge)) = self.disk_gauges.get(mount) else {
                continue;
            };
            free_gauge.set(disk.available_space());
            total_gauge.set(disk.total_space());
        }
    }

    pub fn gather(&self) -> Vec<MetricFamily> {
        self.registry.gather()
    }
}
