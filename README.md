<p align="center">
  <img width="800" src="https://github.com/JakubesP/deta-rust/blob/main/logo.svg?raw=true">
</p>

<br>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/deta_rust.svg)](https://crates.io/crates/deta_rust)

The Deta-Rust is a simple unofficial [Deta](https://www.deta.sh/) SDK for Rust lang.

⚠️ This package is still under active development, so there will be many modifications and improvements ⚠️

Take a look at the examples to get you started quickly. See the [documentation](https://docs.rs/deta_rust/latest/deta_rust/) for details.

Have fun 😀

## Testing

**Unit tests:**
```rust
cargo test --lib
```

**Integration tests:**

Before performing integration tests, you must create a `.env` file in the root directory. It should contain the fields:

```
API_KEY=[...]
TEST_DB_NAME=[...]
TEST_DRIVE_NAME=[...]
```

Then:

```rust
cargo test --test database
cargo test --test drive
```

⚠️ Note, make sure that the database or drive under test does not contain any relevant data ⚠️

## License

Licensed under MIT License.

## Contributions


Contributions would be greatly appreciated.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.

