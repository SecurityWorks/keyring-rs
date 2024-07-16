/*!

# Keyring

This is a cross-platform library that does storage and retrieval of passwords
(or other secrets) in an underlying platform-specific secure store.
A top-level introduction to the library's usage, as well as a small code sample,
may be found in [the library's entry on crates.io](https://crates.io/crates/keyring).
Currently supported platforms are
Linux,
FreeBSD,
OpenBSD,
Windows,
macOS, and iOS.

## Design

This crate implements a very simple, platform-independent concrete object called an _entry_.
Each entry is identified by a <_service name_, _user name_> pair of UTF-8 strings,
optionally augmented by a _target_ string (which can be used to distinguish two entries
that have the same _service name_ and _user name_).
Entries support setting, getting, and forgetting (aka deleting) passwords (UTF-8 strings)
and binary secrets (byte arrays).

Entries provide persistence for their passwords by wrapping credentials held in platform-specific
credential stores.  The implementations of these platform-specific stores are captured
in two types (with associated traits):

- a _credential builder_, represented by the [CredentialBuilder] type
(and [CredentialBuilderApi](credential::CredentialBuilderApi) trait).  Credential
builders are given the identifying information provided for an entry and map
it to the identifying information for a platform-specific credential.
- a _credential_, represented by the [Credential] type
(and [CredentialApi](credential::CredentialApi) trait).  The platform-specific credential
identified by a builder for an entry is what provides the secure storage
for that entry's password/secret.

## Crate-provided Credential Stores

This crate runs on several different platforms, and it provides one
or more implementations of credential stores on each platform.
These implementations work by mapping the data used to identify an entry
to data used to identify platform-specific storage objects.
For example, on macOS, the service and user provided for an entry
are mapped to the service and user attributes that identify a
generic credential in the macOS keychain.

Typically, platform-specific stores (called _keystores_ in this crate)
have a richer model of a credential than
the one used by this crate to identify entries.
These keystores expose their specific model in the
concrete credential objects they use to implement the Credential trait.
In order to allow clients to access this richer model, the Credential trait
has an [as_any](credential::CredentialApi::as_any) method that returns a
reference to the underlying
concrete object typed as [Any](std::any::Any), so that it can be downgraded to
its concrete type.

### Credential store features

Each of the platform-specific credential stores is associated with one or more features.
These features control whether that store is included when the crate is built.  For
example, the macOS Keychain credential store is only included if the `"apple-native"`
feature is specified (and the crate is built with a macOS target).

If no specified credential store features apply to a given platform,
this crate will use the (platform-independent) _mock_ credential store (see below)
on that platform. Specifying multiple credential store features for a given
platform is not supported, and will cause compile-time errors. There are no
default features in this crate: you must specify explicitly which platform-specific
credential stores you intend to use.

Here are the available credential store features:

* `apple-native`: Provides access to the Keychain credential store on macOS and iOS.

* `windows-native`: Provides access to the Windows Credential Store on Windows.

* `linux-native`: Provides access to the `keyutils` storage on Linux.

* `sync-secret-service`: Provides access to the DBus-based
[Secret Service](https://specifications.freedesktop.org/secret-service/latest/)
storage on Linux, FreeBSD, and OpenBSD.  This is a _synchronous_ keystore that provides
support for encrypting secrets when they are transferred across the bus. If you wish
to use this encryption support, additionally specify one (and only one) of the
`crypto-rust` or `crypto-openssl` features (to choose the implementation libraries
used for the encryption). By default, this keystore requires that the DBus library be
installed on the user's machine (and the openSSL library if you specify it for
encryption), but you can avoid this requirement by specifying the `vendored` feature
(which will cause the build to include those libraries statically).

* `async-secret-service`: Provides access to the DBus-based
[Secret Service](https://specifications.freedesktop.org/secret-service/latest/)
storage on Linux, FreeBSD, and OpenBSD.  This is an _asynchronous_ keystore that
always encrypts secrets when they are transferred across the bus. You _must_ specify
both an async runtime feature (either `tokio` or `async-io`) and a cryptographic
implementation (either `crypto-rust` or `crypto-openssl`) when using this
keystore. If you want to use openSSL encryption but those libraries are not
installed on the user's machine, specify the `vendored` feature
to statically link them with the built crate.

## Client-provided Credential Stores

In addition to the platform stores implemented by this crate, clients
are free to provide their own secure stores and use those.  There are
two mechanisms provided for this:

- Clients can give their desired credential builder to the crate
for use by the [Entry::new] and [Entry::new_with_target] calls.
This is done by making a call to [set_default_credential_builder].
The major advantage of this approach is that client code remains
independent of the credential builder being used.

- Clients can construct their concrete credentials directly and
then turn them into entries by using the [Entry::new_with_credential]
call. The major advantage of this approach is that credentials
can be identified however clients want, rather than being restricted
to the simple model used by this crate.

## Mock Credential Store

In addition to the platform-specific credential stores, this crate
always provides a mock credential store that clients can use to
test their code in a platform independent way.  The mock credential
store allows for pre-setting errors as well as password values to
be returned from [Entry] method calls.

## Interoperability with Third Parties

Each of the platform-specific credential stores provided by this crate uses
an underlying store that may also be used by modules written
in other languages.  If you want to interoperate with these third party
credential writers, then you will need to understand the details of how the
target, service, and user of this crate's generic model
are used to identify credentials in the platform-specific store.
These details are in the implementation of this crate's secure-storage
modules, and are documented in the headers of those modules.

(_N.B._ Since the included credential store implementations are platform-specific,
you may need to use the Platform drop-down on [docs.rs](https://docs.rs/keyring) to
view the storage module documentation for your desired platform.)

## Caveats

This module expects passwords to be UTF-8 encoded strings,
so if a third party has stored an arbitrary byte string
then retrieving that as a password will return a
[BadEncoding](Error::BadEncoding) error.
The returned error will have the raw bytes attached,
so you can access them, but you can also just fetch
them directly using [Entry::get_secret] rather than
[Entry:get_password].

While this crate's code is thread-safe, the underlying credential
stores may not handle access from different threads reliably.
In particular, accessing the same credential
from multiple threads at the same time can fail, especially on
Windows and Linux, because the accesses may not be serialized in the same order
they are made. And for RPC-based credential stores such as the dbus-based Secret
Service, accesses from multiple threads (and even the same thread very quickly)
are not recommended, as they may cause the RPC mechanism to fail.
 */
pub use credential::{Credential, CredentialBuilder};
pub use error::{Error, Result};

pub mod mock;

//
// no duplicate keystores on any platform
//
#[cfg(any(
    all(feature = "linux-native", feature = "sync-secret-service"),
    all(feature = "linux-native", feature = "async-secret-service"),
    all(feature = "sync-secret-service", feature = "async-secret-service")
))]
compile_error!("You can enable at most one keystore per target architecture");

//
// Pick the *nix keystore
//

#[cfg(all(target_os = "linux", feature = "linux-native"))]
pub mod keyutils;
#[cfg(all(target_os = "linux", feature = "linux-native"))]
use keyutils as default;

#[cfg(all(
    any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"),
    any(feature = "sync-secret-service", feature = "async-secret-service")
))]
pub mod secret_service;
#[cfg(all(
    any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"),
    any(feature = "sync-secret-service", feature = "async-secret-service")
))]
use secret_service as default;

#[cfg(all(
    target_os = "linux",
    not(any(
        feature = "linux-native",
        feature = "sync-secret-service",
        feature = "async-secret-service"
    ))
))]
use mock as default;
#[cfg(all(
    any(target_os = "freebsd", target_os = "openbsd"),
    not(any(feature = "sync-secret-service", feature = "async-secret-service"))
))]
use mock as default;

//
// pick the Apple keystore
//
#[cfg(all(target_os = "macos", feature = "apple-native"))]
pub mod macos;
#[cfg(all(target_os = "macos", feature = "apple-native"))]
use macos as default;
#[cfg(all(target_os = "macos", not(feature = "apple-native")))]
use mock as default;

#[cfg(all(target_os = "ios", feature = "apple-native"))]
pub mod ios;
#[cfg(all(target_os = "ios", feature = "apple-native"))]
use ios as default;
#[cfg(all(target_os = "ios", not(feature = "apple-native")))]
use mock as default;

//
// pick the Windows keystore
//

#[cfg(all(target_os = "windows", feature = "windows-native"))]
pub mod windows;
#[cfg(all(target_os = "windows", not(feature = "windows-native")))]
use mock as default;
#[cfg(all(target_os = "windows", feature = "windows-native"))]
use windows as default;

#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "macos",
    target_os = "ios",
    target_os = "windows",
)))]
use mock as default;

pub mod credential;
pub mod error;

#[derive(Default, Debug)]
struct EntryBuilder {
    inner: Option<Box<CredentialBuilder>>,
}

static DEFAULT_BUILDER: std::sync::RwLock<EntryBuilder> =
    std::sync::RwLock::new(EntryBuilder { inner: None });

/// Set the credential builder used by default to create entries.
///
/// This is really meant for use by clients who bring their own credential
/// store and want to use it everywhere.  If you are using multiple credential
/// stores and want precise control over which credential is in which store,
/// then use [new_with_credential](Entry::new_with_credential).
///
/// This will block waiting for all other threads currently creating entries
/// to complete what they are doing. It's really meant to be called
/// at app startup before you start creating entries.
pub fn set_default_credential_builder(new: Box<CredentialBuilder>) {
    let mut guard = DEFAULT_BUILDER
        .write()
        .expect("Poisoned RwLock in keyring-rs: please report a bug!");
    guard.inner = Some(new);
}

fn build_default_credential(target: Option<&str>, service: &str, user: &str) -> Result<Entry> {
    static DEFAULT: std::sync::OnceLock<Box<CredentialBuilder>> = std::sync::OnceLock::new();
    let guard = DEFAULT_BUILDER
        .read()
        .expect("Poisoned RwLock in keyring-rs: please report a bug!");
    let builder = guard
        .inner
        .as_ref()
        .unwrap_or_else(|| DEFAULT.get_or_init(|| default::default_credential_builder()));
    let credential = builder.build(target, service, user)?;
    Ok(Entry { inner: credential })
}

#[derive(Debug)]
pub struct Entry {
    inner: Box<Credential>,
}

impl Entry {
    /// Create an entry for the given service and user.
    ///
    /// The default credential builder is used.
    pub fn new(service: &str, user: &str) -> Result<Entry> {
        build_default_credential(None, service, user)
    }

    /// Create an entry for the given target, service, and user.
    ///
    /// The default credential builder is used.
    pub fn new_with_target(target: &str, service: &str, user: &str) -> Result<Entry> {
        build_default_credential(Some(target), service, user)
    }

    /// Create an entry that uses the given platform credential for storage.
    pub fn new_with_credential(credential: Box<Credential>) -> Entry {
        Entry { inner: credential }
    }

    /// Set the password for this entry.
    ///
    /// Can return an [Ambiguous](Error::Ambiguous) error
    /// if there is more than one platform credential
    /// that matches this entry.  This can only happen
    /// on some platforms, and then only if a third-party
    /// application wrote the ambiguous credential.
    pub fn set_password(&self, password: &str) -> Result<()> {
        self.inner.set_password(password)
    }

    /// Set the secret for this entry.
    ///
    /// Can return an [Ambiguous](Error::Ambiguous) error
    /// if there is more than one platform credential
    /// that matches this entry.  This can only happen
    /// on some platforms, and then only if a third-party
    /// application wrote the ambiguous credential.
    pub fn set_secret(&self, secret: &[u8]) -> Result<()> {
        self.inner.set_secret(secret)
    }

    /// Retrieve the password saved for this entry.
    ///
    /// Returns a [NoEntry](Error::NoEntry) error if there isn't one.
    ///
    /// Can return an [Ambiguous](Error::Ambiguous) error
    /// if there is more than one platform credential
    /// that matches this entry.  This can only happen
    /// on some platforms, and then only if a third-party
    /// application wrote the ambiguous credential.
    pub fn get_password(&self) -> Result<String> {
        self.inner.get_password()
    }

    /// Retrieve the secret saved for this entry.
    ///
    /// Returns a [NoEntry](Error::NoEntry) error if there isn't one.
    ///
    /// Can return an [Ambiguous](Error::Ambiguous) error
    /// if there is more than one platform credential
    /// that matches this entry.  This can only happen
    /// on some platforms, and then only if a third-party
    /// application wrote the ambiguous credential.
    pub fn get_secret(&self) -> Result<Vec<u8>> {
        self.inner.get_secret()
    }

    /// Delete the underlying credential for this entry.
    ///
    /// Returns a [NoEntry](Error::NoEntry) error if there isn't one.
    ///
    /// Can return an [Ambiguous](Error::Ambiguous) error
    /// if there is more than one platform credential
    /// that matches this entry.  This can only happen
    /// on some platforms, and then only if a third-party
    /// application wrote the ambiguous credential.
    ///
    /// Note: This does _not_ affect the lifetime of the [Entry]
    /// structure, which is controlled by Rust.  It only
    /// affects the underlying credential store.
    pub fn delete_credential(&self) -> Result<()> {
        self.inner.delete_credential()
    }

    /// Return a reference to this entry's wrapped credential.
    ///
    /// The reference is of the [Any](std::any::Any) type, so it can be
    /// downgraded to a concrete credential object.  The client must know
    /// what type of concrete object to cast to.
    pub fn get_credential(&self) -> &dyn std::any::Any {
        self.inner.as_any()
    }
}

#[cfg(doctest)]
doc_comment::doctest!("../README.md", readme);

#[cfg(test)]
/// There are no actual tests in this module.
/// Instead, it contains generics that each keystore invokes in their tests,
/// passing their store-specific parameters for the generic ones.
//
// Since iOS doesn't use any of these generics, we allow dead code.
#[allow(dead_code)]
mod tests {
    use super::{credential::CredentialApi, Entry, Error, Result};
    use rand::Rng;

    /// Create a platform-specific credential given the constructor, service, and user
    pub fn entry_from_constructor<F, T>(f: F, service: &str, user: &str) -> Entry
    where
        F: FnOnce(Option<&str>, &str, &str) -> Result<T>,
        T: 'static + CredentialApi + Send + Sync,
    {
        match f(None, service, user) {
            Ok(credential) => Entry::new_with_credential(Box::new(credential)),
            Err(err) => {
                panic!("Couldn't create entry (service: {service}, user: {user}): {err:?}")
            }
        }
    }

    /// A basic round-trip unit test given an entry and a password.
    pub fn test_round_trip(case: &str, entry: &Entry, in_pass: &str) {
        entry
            .set_password(in_pass)
            .unwrap_or_else(|err| panic!("Can't set password for {case}: {err:?}"));
        let out_pass = entry
            .get_password()
            .unwrap_or_else(|err| panic!("Can't get password for {case}: {err:?}"));
        assert_eq!(
            in_pass, out_pass,
            "Passwords don't match for {case}: set='{in_pass}', get='{out_pass}'",
        );
        entry
            .delete_credential()
            .unwrap_or_else(|err| panic!("Can't delete password for {case}: {err:?}"));
        let password = entry.get_password();
        assert!(
            matches!(password, Err(Error::NoEntry)),
            "Read deleted password for {case}",
        );
    }

    /// A basic round-trip unit test given an entry and a password.
    pub fn test_round_trip_secret(case: &str, entry: &Entry, in_secret: &[u8]) {
        entry
            .set_secret(in_secret)
            .unwrap_or_else(|err| panic!("Can't set secret for {case}: {err:?}"));
        let out_secret = entry
            .get_secret()
            .unwrap_or_else(|err| panic!("Can't get secret for {case}: {err:?}"));
        assert_eq!(
            in_secret, &out_secret,
            "Passwords don't match for {case}: set='{in_secret:?}', get='{out_secret:?}'",
        );
        entry
            .delete_credential()
            .unwrap_or_else(|err| panic!("Can't delete password for {case}: {err:?}"));
        let password = entry.get_secret();
        assert!(
            matches!(password, Err(Error::NoEntry)),
            "Read deleted password for {case}",
        );
    }

    /// When tests fail, they leave keys behind, and those keys
    /// have to be cleaned up before the tests can be run again
    /// in order to avoid bad results.  So it's a lot easier just
    /// to have tests use a random string for key names to avoid
    /// the conflicts, and then do any needed cleanup once everything
    /// is working correctly.  So we export this function for tests to use.
    pub fn generate_random_string_of_len(len: usize) -> String {
        // from the Rust Cookbook:
        // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
        use rand::{distributions::Alphanumeric, thread_rng, Rng};
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    pub fn generate_random_string() -> String {
        generate_random_string_of_len(30)
    }

    pub fn test_empty_service_and_user<F>(f: F)
    where
        F: Fn(&str, &str) -> Entry,
    {
        let name = generate_random_string();
        let in_pass = "doesn't matter";
        test_round_trip("empty user", &f(&name, ""), in_pass);
        test_round_trip("empty service", &f("", &name), in_pass);
        test_round_trip("empty service & user", &f("", ""), in_pass);
    }

    pub fn test_missing_entry<F>(f: F)
    where
        F: FnOnce(&str, &str) -> Entry,
    {
        let name = generate_random_string();
        let entry = f(&name, &name);
        assert!(
            matches!(entry.get_password(), Err(Error::NoEntry)),
            "Missing entry has password"
        )
    }

    pub fn test_empty_password<F>(f: F)
    where
        F: FnOnce(&str, &str) -> Entry,
    {
        let name = generate_random_string();
        let entry = f(&name, &name);
        test_round_trip("empty password", &entry, "");
    }

    pub fn test_round_trip_ascii_password<F>(f: F)
    where
        F: FnOnce(&str, &str) -> Entry,
    {
        let name = generate_random_string();
        let entry = f(&name, &name);
        test_round_trip("ascii password", &entry, "test ascii password");
    }

    pub fn test_round_trip_non_ascii_password<F>(f: F)
    where
        F: FnOnce(&str, &str) -> Entry,
    {
        let name = generate_random_string();
        let entry = f(&name, &name);
        test_round_trip("non-ascii password", &entry, "このきれいな花は桜です");
    }

    pub fn test_round_trip_random_secret<F>(f: F)
    where
        F: FnOnce(&str, &str) -> Entry,
    {
        let name = generate_random_string();
        let entry = f(&name, &name);
        let mut secret: [u8; 16] = [0; 16];
        rand::rngs::OsRng.fill(&mut secret);
        test_round_trip_secret("non-ascii password", &entry, &secret);
    }

    pub fn test_update<F>(f: F)
    where
        F: FnOnce(&str, &str) -> Entry,
    {
        let name = generate_random_string();
        let entry = f(&name, &name);
        test_round_trip("initial ascii password", &entry, "test ascii password");
        test_round_trip(
            "updated non-ascii password",
            &entry,
            "このきれいな花は桜です",
        );
    }
}
