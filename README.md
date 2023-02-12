# Program Ingester

## Workspace members

- [cli](./cli/) - end application
- [program_ingester](./program_ingester/) - library (that can be used in applications like CLI, Web Server, etc...)

## Test

```sh
cargo test
```

## Run examples

> **Note**: I've used examples to test small bits of code for building the library. Usually I use examples for showing how to use the library in various ways.

```sh
cargo run --example simple_tree
```

## Generate docs:

> **Note**: Rust docs are awesome. Cargo can compile example code in comments to make sure they are correct.

```sh
cargo doc --no-deps --lib --document-private-items # add --open to open in a new browser tab
```
