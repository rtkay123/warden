# warden

An experimental rewrite of the core Tazama platform processors in Rust.

[![codecov](https://codecov.io/github/rtkay123/warden/graph/badge.svg?token=D2N2885O77)](https://codecov.io/github/rtkay123/warden)
![GitHub License](https://img.shields.io/github/license/rtkay123/warden)
[![ci](https://github.com/rtkay123/warden/actions/workflows/ci.yaml/badge.svg)](https://github.com/rtkay123/warden/actions/workflows/ci.yaml)
![status: Experimental](https://img.shields.io/badge/status-experimental-orange)

## Project Structure

| Directory   | Description                                     |
| ----------- | ----------------------------------------------- |
| `/crates`   | Core distributed applications                   |
| `/lib`      | Shared utilities and libraries                  |
| `/contrib/` | Goodies which may be useful for contributors    |
| `/docs/`    | Detailed architecture and design documentation  |


## Building

### Dependencies

- [`protoc`] - protobuf compiler
- [`cargo`] - Rust's package manager
- [`rustc`] - MSRV 1.91.0
- [docker] - (optional)

> [!TIP]
> If planning to run on docker, you do not need the other 3 dependencies

Clone the repository:

```sh
git clone https://github.com/rtkay123/warden.git
cd warden
```

Warden also depends on (protobuf) types from [googleapis], which is declared
as a submodule. Update that before proceeding.

```sh
git submodule update --init --depth 1 --recommend-shallow
```

You may then proceed with building the platform through [`cargo`]:

```sh
cargo build
```

### Configure Applications

Each application requires a configuration file in TOML format. Before running,
it could be useful to checkout what each application expects through the config.

In the [crates] directory, there are subfolders representing each binary.
Each of these subfolders contains a `.toml` file used as the default config.

### External Services

Warden provides sample Docker [compose] files for the services used by the
platform at runtime:

- `compose.yaml`: contains core components which are needed for the platform to
  run
- `compose.monitoring.yaml`: contains **optional** tools for distributed for
  monitoring and distributed tracing

> [!NOTE]
> The applications may still make noise about monitoring tools not being
available in logs, but these are safe to ignore

> [!TIP]
> Silence the noise by adjusting your log level in the configuration

Each application requires a configuration file in TOML format. Before running,
it could be useful to checkout what each application expects through the config.

In the [crates](./crates/) directory, there are subfolders representing each binary.
Each of these subfolders contains a `.toml` file used as the default config.


### Run Applications

#### Native

An example for the configuration API:
```sh
cargo run -p warden-config
```

#### Docker

Build and run the images. An example for the configuration API:

```sh
docker build -f crates/configuration/Dockerfile -t warden-config:latest .
docker run -p 1304:1304 warden-config:latest 
```

> [!TIP]
> Some processors leverage conditional compilation to toggle additional features.
Checkout each processor's documentation to see what else it is capable of

### Testing End-to-End

An example for using [Bruno] for API testing is avaiable through the [sample collection and environment](./contrib/bruno/)
. This collection supplies initial configuration data and can be used for triggering the end-to-end flow

> [!IMPORTANT]
> You can run the applications in any order **after** the pseudonyms and configuration
service are running

## License

This project is licensed under AGPL-3.0

## Disclaimer

This project is not affiliated with, or endorsed by Tazama. It is an
independent rewrite created for educational and experimental purposes only.
All contributions to the original project should go to the
[official repository](https://github.com/tazama-lf).

[Tazama]: https://tazama.org
[googleapis]: https://github.com/googleapis/googleapis
[`cargo`]: https://www.rust-lang.org/tools/install
[`protoc`]: https://protobuf.dev/installation/
[`rustc`]: https://www.rust-lang.org/tools/install
[compose]: https://docs.docker.com/compose/
[docker]: https://docs.docker.com
