# sysinfo-exporter

A simple service collecting system metrics like CPU, memory, disk space and network usage and pushing them to a [Prometheus](https://prometheus.io/docs/introduction/overview/) instance using the [write API](https://prometheus.io/blog/2019/10/10/remote-read-meets-streaming/#remote-apis). Specifically, this is useful if you want to send simple system metrics from servers, which are not or shall not be accessible via a web API.

> [!NOTE]
> This service has a very limited set of metrics used for very simple system monitoring. If you are looking for a more comprehensive solution, please take a look at the following projects.
>
> - [Prometheus Node Exporter](https://github.com/prometheus/node_exporter)
> - [Grafana Alloy](https://github.com/grafana/alloy)

## Installation

### Debian Package

On Debian based systems, you can download the Debian package from the [Releases Page](https://github.com/zekroTJA/sysinfo-exporter/releases) for your system and install it using dpkg.

```
dpkg -i sysinfo-exporter-*.deb
```

After that, open the configuration file placed in `/etc/system-exporter/config.toml` and enter the Prometheus endpoint and other desired configuration parameters.

### Manual

Simply download the latest binary form the [Releases Page](https://github.com/zekroTJA/sysinfo-exporter/releases) for your system architecture and install it on your system.

After that, choose a way for configuring the service.

## Configuration

The configuration can either be passed via endironment variables (prefixed with `SYSEXP_`) or via a TOML config file passed by the `--config` parameter.

Here you can find an example configuration.

```toml
# The write API endpoint of the target prometheus instance.
endpoint = "https://prometheus.example.com/api/v1/write"

# When the target prometheus instance requires authentication,
# here you can specify the type and credentials.
# [auth.basic]
# username = ""
# password = ""
# [auth.bearer]
# token = ""

# The log level of the application logger.
# Besides the global log level, you can specify levels for filters.
# Here you can find more information:
# https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives
loglevel = "info"

# The interval (in seconds) in which system information data is
# refreshed and pushed to the prometheus instance.
interval_seconds = 15

# You can specify additional labels, which are added to each
# metric sent from this instance.
[labels]
location = "home-1"
```
