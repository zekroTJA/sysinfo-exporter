use crate::{collector::Collector, config::Auth};
use anyhow::Result;
use base64::{Engine, prelude::BASE64_STANDARD};
use prometheus_reqwest_remote_write::WriteRequest;
use reqwest::{Client, header::HeaderMap};
use std::{collections::HashMap, time::Duration};
use sysinfo::System;
use tokio::{sync::Mutex, time::sleep};

pub struct Metrics {
    client: Client,
    endpoint: String,
    collector: Mutex<Collector>,
    labels: Vec<(String, String)>,
}

impl Metrics {
    pub fn new<S: Into<String>>(
        endpoint: S,
        auth: Option<Auth>,
        labels: Option<HashMap<String, String>>,
    ) -> Result<Self> {
        let endpoint = endpoint.into();

        let collector = Mutex::new(Collector::new()?);

        let mut default_headers = HeaderMap::new();

        match auth {
            Some(Auth::Basic { username, password }) => {
                let basic_auth = BASE64_STANDARD.encode(format!("{username}:{password}"));
                default_headers.insert("Authorization", format!("Basic {basic_auth}").parse()?);
            }
            Some(Auth::Bearer { token }) => {
                default_headers.insert("Authorization", format!("Bearer {token}").parse()?);
            }
            None => {}
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .default_headers(default_headers)
            .build()?;

        let mut labels = labels.unwrap_or_default();

        if let Some(hostname) = get_hostname() {
            if !labels.contains_key("hostname") {
                labels.insert("hostname".into(), hostname);
            }
        }

        Ok(Self {
            client,
            collector,
            endpoint,
            labels: labels.into_iter().collect(),
        })
    }

    async fn sollect_and_send_metrics(&self) -> Result<()> {
        let mut collector = self.collector.lock().await;
        collector.collect();

        let write_req =
            WriteRequest::from_metric_families(collector.gather(), Some(self.labels.clone()))
                .map_err(|e| anyhow::anyhow!("failed getting write request: {e}"))?;

        let http_req = write_req.build_http_request(
            self.client.clone(),
            &self.endpoint,
            "sysinfo-exporter",
        )?;

        self.client.execute(http_req).await?.error_for_status()?;

        Ok(())
    }

    pub async fn start_schedule(&self, interval: Duration) {
        tracing::info!("Starting metrics collection scheduler ...");

        loop {
            match self.sollect_and_send_metrics().await {
                Ok(_) => tracing::info!("Metrics collected and sent"),
                Err(err) => tracing::error!("Metrics collection failed: {err}"),
            };
            sleep(interval).await;
        }
    }
}

fn get_hostname() -> Option<String> {
    System::host_name()
}
