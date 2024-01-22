# Artifex - Web client (using Yew)

This is a web application client to interact with an Artifex server over gPRC,
using [Yew][yew] framework.

# Setup

This project uses [trunk][trunk] as its build system. Execute the following
command to install it:

```sh
cargo install --locked trunk
```

# Build

To build and serve the application using a local server, execute:

```sh
trunk serve index.html
```

To build the application, execute:

```sh
trunk build index.html
```

The files to serve will be available in the ``dist`` directory.

# License

Copyright (c) 2023 Eric Le Bihan

This program is distributed under the terms of the MIT License.

See the [LICENSE](LICENSE) file for license details.

[trunk]: https://trunkrs.dev/
[yew]: https://yew.rs/
