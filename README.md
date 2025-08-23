# warden

An experimental rewrite of the Tazama platform in Rust.

[![codecov](https://codecov.io/github/rtkay123/warden/graph/badge.svg?token=D2N2885O77)](https://codecov.io/github/rtkay123/warden)
[![ci](https://github.com/rtkay123/warden/actions/workflows/ci.yaml/badge.svg)](https://github.com/rtkay123/warden/actions/workflows/ci.yaml)

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
- [`rustc`] - MSRV 1.89

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
