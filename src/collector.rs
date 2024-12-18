use anyhow::Result;
use prometheus::{Gauge, Opts, Registry, proto::MetricFamily};
use sysinfo::System;

pub struct Collector {
    registry: Registry,

    system: System,

    cpu_usage: Gauge,
}

impl Collector {
    pub fn new() -> Result<Self> {
        let system = System::new_all();

        let cpu_usage = Gauge::with_opts(Opts::new("sysinfo_cpu_usage", "Average CPU usage"))?;

        let registry = Registry::new();
        registry.register(Box::new(cpu_usage.clone()))?;

        Ok(Self {
            registry,
            system,
            cpu_usage,
        })
    }

    pub fn collect(&mut self) {
        self.system.refresh_all();
        self.cpu_usage.set(self.system.global_cpu_usage() as f64);
    }

    pub fn gather(&self) -> Vec<MetricFamily> {
        self.registry.gather()
    }
}
