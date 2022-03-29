# Configuration

Pages is highly configurable. Configuration is applied/merged in the
following order:

1. `/etc/static-pages/config.toml`
2. `./config/default.toml`
3. path to configuration file passed in via `PAGES_CONFIG`
4. environment variables.

So if `/etc/static-pages/config.toml` says Pages must listen on port
`4000` and environment variable or `PAGES_CONFIG` file say it should
listen on port `5000`, Pages will listen on `5000`.

## Setup

### Environment variables

Setting environment variables are optional. The configuration files have
all the necessary parameters listed. By setting environment variables,
you will be overriding the values set in the configuration files.

### General

| Name                  | Value                                    |
| --------------------- | ---------------------------------------- |
| `PAGES_CONFIG`        | Path to configuration file               |
| `PAGES__SOURCE__CODE` | Link to the source code of this instance |

#### Server

| Name                                    | Value                                                                                                                                                             |
| --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `PAGES__SERVER__PORT`                   | The port on which you want Pages to listen to                                                                                                                     |
| `PORT`(overrides `PAGES__SERVER__PORT`) | The port on which you want Pages to listen to                                                                                                                     |
| `PAGES__SERVER__IP`                     | The IP address on which you want Pages to listen to                                                                                                               |
| `PAGES__SERVER__WORKERS`                | The number of worker threads that must be spun up by the Pages server. Minimum of two threads are advisable for top async performance but can work with one also. |
