# Artifex - Server

This is the server which exposes the features of the Artifex engine over gRPC.

## Usage examples

Interacting with the server can be done using [grpcurl](https://github.com/fullstorydev/grpcurl).

List services:

```
➜ grpcurl -plaintext localhost:50051 list
artifex.Artifex
grpc.reflection.v1alpha.ServerReflection
```

List methods for a service:

```
➜ grpcurl -plaintext localhost:50051 list artifex.Artifex
artifex.Artifex.Execute
artifex.Artifex.Inspect
artifex.Artifex.Upgrade
```

Call method `Inspect`:

```
➜ grpcurl -plaintext localhost:50051 artifex.Artifex/Inspect
{
  "kernelVersion": "5.19.8-200.fc36.x86_64"
}
```

Get details on how to invoke `Execute` method:

```
➜ grpcurl -plaintext localhost:50051 describe artifex.Artifex.Execute
artifex.Artifex.Execute is a method:
// Execute a command on a machine
rpc Execute ( .artifex.ExecuteRequest ) returns ( .artifex.ExecuteReply );
```

Get details on `Execute` argument and result:

```
➜ grpcurl -plaintext localhost:50051 describe artifex.ExecuteRequest
artifex.ExecuteRequest is a message:
message ExecuteRequest {
  // The command to execute on the machine.
  string command = 1;
➜ grpcurl -plaintext localhost:50051 describe artifex.ExecuteReply
artifex.ExecuteReply is a message:
// Reply from the execution of a command
message ExecuteReply {
  // Standard output
  string stdout = 1;
  // Standard error
  string stderr = 2;
  // Command exit code
  int32 code = 3;
}
```

Call method `Execute` with data:

```
➜ echo '{ "command": "uname -a" }' | grpcurl -d @ -plaintext localhost:50051 artifex.Artifex/Execute
{
  "stdout": "Linux itchy 6.0.8-300.fc37.x86_64 #1 SMP PREEMPT_DYNAMIC Fri Nov 11 15:09:04 UTC 2022 x86_64 x86_64 x86_64 GNU/Linux\n"
}
```

# License

Copyright (c) 2022 Eric Le Bihan

This program is distributed under the terms of the MIT License.

See the [LICENSE](LICENSE) file for license details.
