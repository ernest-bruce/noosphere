use std::{borrow::Borrow, fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};

/// A DID, aka a Decentralized Identifier, is a string that can be parsed and
/// resolved into a so-called DID Document, usually in order to obtain PKI
/// details related to a particular user or process.
///
/// See: https://en.wikipedia.org/wiki/Decentralized_identifier
/// See: https://www.w3.org/TR/did-core/
#[repr(transparent)]
#[derive(Default, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Did(pub String);

impl Deref for Did {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for Did {
    fn from(value: &str) -> Self {
        Did(value.to_owned())
    }
}

impl From<String> for Did {
    fn from(value: String) -> Self {
        Did(value)
    }
}

impl From<Did> for String {
    fn from(value: Did) -> Self {
        value.0
    }
}

impl Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// A JWT, aka a JSON Web Token, is a specialized string-encoding of a
/// particular format of JSON and an associated signature, commonly used for
/// authorization flows on the web, but notably also used by the UCAN spec.
///
/// See: https://jwt.io/
/// See: https://ucan.xyz/
#[repr(transparent)]
pub struct Jwt(pub String);

impl Deref for Jwt {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Jwt {
    fn from(value: String) -> Self {
        Jwt(value)
    }
}

/// A BIP39-compatible mnemonic phrase that represents the data needed to
/// recover the private half of a cryptographic key pair.
///
/// See: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
#[repr(transparent)]
pub struct Mnemonic(pub String);

impl Deref for Mnemonic {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Mnemonic {
    fn from(value: String) -> Self {
        Mnemonic(value)
    }
}
