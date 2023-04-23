# Artifex - server and client to do stuff remotely on machine

This project, *FOR EDUCATION, NOT PRODUCTION*, provides:

- a server which can perform some tasks (report operating system version,
  execute arbitrary command, simulate operating system upgrade) and exposes a
  RPC interface over [gRPC][GRPC]
- a command line client to interact with the server.

## Build instructions

To build and run the server, execute:

```sh
cargo run -p artifex-server
```

To build and run the client, execute:

```sh
cargo run -p artifex-client
```

# License

Copyright © 2022-2023 Eric Le Bihan

This program is distributed under the terms of the MIT License.

See the [LICENSE](LICENSE) file for license details.

[GRPC]: https://grpc.io
