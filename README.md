## Keyring-rs
[![CI](https://github.com/hwchen/keyring-rs/workflows/ci/badge.svg)](https://github.com/hwchen/keyring-rs/actions?query=workflow%3Aci)
[![Crates.io](https://img.shields.io/crates/v/keyring.svg?style=flat-square)](https://crates.io/crates/keyring)
[![API Documentation on docs.rs](https://docs.rs/keyring/badge.svg)](https://docs.rs/keyring)

A cross-platorm library and utility to manage passwords.

Online [docs](https://docs.rs/keyring) are currently limited to linux, as cross-platform autogenerated docs are not a thing yet. For osx or windows, try `cargo doc -p keyring --open`.

Published on [crates.io](https://crates.io/crates/keyring)

## Usage

__Currently supports Linux, macOS, and Windows.__ Please file issues if you have any problems or bugs!

To use this library in your project add the following to your `Cargo.toml` file:

```
[dependencies]
keyring = "0.10.1"
```

This will give you access to the `keyring` crate in your code. Now you can use
the `new` function to get an instance of the `Keyring` struct. The `new`
function expects a `service` name and an `username` with which it accesses
the password.

You can get a password from the OS keyring with the `get_password` function.

```rust,no_run
extern crate keyring;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let service = "my_application_name";
  let username = "username";

  let keyring = keyring::Keyring::new(&service, &username);

  let password = keyring.get_password()?;
  println!("The password is '{}'", password);

  Ok(())
}
```

Passwords can also be added to the keyring using the `set_password` function.

```rust,no_run
extern crate keyring;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let service = "my_application_name";
  let username = "username";

  let keyring = keyring::Keyring::new(&service, &username);

  let password = "topS3cr3tP4$$w0rd";
  keyring.set_password(&password)?;

  let password = keyring.get_password()?;
  println!("The password is '{}'", password);

  Ok(())
}
```

And they can be deleted with the `delete_password` function.

```rust,no_run
extern crate keyring;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let service = "my_application_name";
  let username = "username";

  let keyring = keyring::Keyring::new(&service, &username);

  keyring.delete_password()?;

  println!("The password has been deleted");

  Ok(())
}
```

On macOS, keychain object from specific path can be opened using `Keyring::use_keychain` which gives the flexibility to open non-default keychains. Note that this is currently feature-gated, and is considered unstable, and is subject to change without a semver major version change.

In Cargo.toml, you need to turn the feature on:
```toml
keyring = { version = "0.10.0", features = ["macos-specify-keychain"] }
```

```rust,no_run
extern crate keyring;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let service = "my_application_name";
  let username = "username";

  let keyring = keyring::Keyring::use_keychain(Path::new("/Library/Keychains/System.keychain"), &service, &username);

  let password = "topS3cr3tP4$$w0rd";
  keyring.set_password(&password)?;

  let password = keyring.get_password()?;
  println!("The password is '{}'", password);

  Ok(())
}
```

## Errors

The `get_password`, `set_password` and `delete_password` functions return a
`Result` which, if the operation was unsuccessful, can yield a `KeyringError`.

The `KeyringError` struct implements the `error::Error` and `fmt::Display`
traits, so it can be queried for a cause and an description using methods of
the same name.

## Caveats

### Linux

* The application name is hardcoded to be `rust-keyring`.

## Dev Notes

* If you're running tests, please use `RUST_TEST_THREADS=1 cargo test`
* for TravisCI, osx builds and tests, but linux only builds. Need to figure out how to mock secret service.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributors
Thanks to the following for helping make this library better, whether through contributing code, discussion, or bug reports!

- @dario23
- @dten
- @jasikpark
- @jonathanmorley
- @lexxvir
- @Phrohdoh
- @Rukenshia
- @samuela
- @stankec
- @steveatinfincia
- @bhkaminski
- @MaikKlein

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

