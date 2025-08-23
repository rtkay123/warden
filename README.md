# warden

An experimental rewrite of the Tazama platform in Rust.

# Building

## Dependencies

- [`protoc`]
- [`cargo`]
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

## Running

Warden provides sample Docker [compose] files for the services used by the
platform at runtime:

- `compose.yaml`: contains core components which are needed for the platform to
  run
- `compose.monitoring.yaml`: contains **optional** tools for distributed for
  monitoring and distributed tracing

Note: the applications may still make noise about monitoring tools not being
available in logs, but these are safe to ignore

# Disclaimer

This project is not affiliated with, or endorsed by Tazama. It is an
independent rewrite created for educational and experimental purposes only.
All contributions to the original project should go to the official repository.

[Tazama]: https://tazama.org
[googleapis]: https://github.com/googleapis/googleapis
[`cargo`]: https://github.com/googleapis/googleapis
[`protoc`]: https://github.com/googleapis/googleapis
[`rustc`]: https://github.com/googleapis/googleapis
