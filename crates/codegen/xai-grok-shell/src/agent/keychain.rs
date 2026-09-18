//! Resolve a model API key from the OS credential store.
//!
//! The secret is read at call time and never written back to config or logs.
//! A missing or empty item is the same as an unset `env_key`: that source
//! produces no credential and later fallbacks may still apply.

use std::cell::RefCell;

use serde::{Deserialize, Serialize};

/// Default OS keychain service. Keep `security add-generic-password -s` in sync.
/// Users can override with `keychain_service` but are not recommended to.
pub const DEFAULT_KEYCHAIN_SERVICE: &str = "grok";

/// Keychain item locator from `[model.<id>] keychain_account` (and optional `keychain_service`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeychainRef {
    pub service: String,
    pub account: String,
}

impl KeychainRef {
    /// `keychain_account` must be non-empty after trim. `keychain_service` defaults to
    /// [`DEFAULT_KEYCHAIN_SERVICE`] when unset or blank.
    pub(crate) fn from_parts(service: Option<&str>, account: Option<&str>) -> Option<Self> {
        let account = account.map(str::trim).filter(|s| !s.is_empty())?;
        let service = service
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_KEYCHAIN_SERVICE);
        Some(Self {
            service: service.to_owned(),
            account: account.to_owned(),
        })
    }

    pub(crate) fn resolve_value(&self) -> Option<String> {
        self.resolve_value_with(lookup_secret)
    }

    /// Testable resolve with an injected backend.
    pub(crate) fn resolve_value_with(
        &self,
        mut lookup: impl FnMut(&str, &str) -> Option<String>,
    ) -> Option<String> {
        lookup(&self.service, &self.account).filter(|secret| !secret.trim().is_empty())
    }
}

/// Read a generic password from the platform store.
///
/// macOS Keychain and Windows Credential Manager are compiled in on those
/// targets. Linux has no Secret Service backend here (it would need libdbus).
/// Any error — including "no such item" and "no store available" — is `None`.
pub(crate) fn lookup_secret(service: &str, account: &str) -> Option<String> {
    #[cfg(test)]
    {
        if let Some(lookup) = KEYCHAIN_LOOKUP_OVERRIDE.with(|slot| *slot.borrow()) {
            return lookup(service, account);
        }
    }
    platform_lookup_secret(service, account)
}

fn platform_lookup_secret(service: &str, account: &str) -> Option<String> {
    let entry = match keyring::Entry::new(service, account) {
        Ok(entry) => entry,
        Err(error) => {
            tracing::debug!(
                service,
                account,
                error = %error,
                "keychain entry create failed"
            );
            return None;
        }
    };
    match entry.get_password() {
        Ok(secret) if !secret.trim().is_empty() => Some(secret),
        Ok(_) => None,
        Err(keyring::Error::NoEntry) => None,
        Err(error) => {
            tracing::debug!(
                service,
                account,
                error = %error,
                "keychain lookup failed"
            );
            None
        }
    }
}

thread_local! {
    static KEYCHAIN_LOOKUP_OVERRIDE: RefCell<Option<fn(&str, &str) -> Option<String>>> =
        const { RefCell::new(None) };
}

/// Install a process-thread fake keychain for the lifetime of the returned guard.
#[cfg(test)]
pub(crate) fn override_keychain_lookup(
    lookup: fn(&str, &str) -> Option<String>,
) -> KeychainLookupGuard {
    KEYCHAIN_LOOKUP_OVERRIDE.with(|slot| *slot.borrow_mut() = Some(lookup));
    KeychainLookupGuard
}

#[cfg(test)]
pub(crate) struct KeychainLookupGuard;

#[cfg(test)]
impl Drop for KeychainLookupGuard {
    fn drop(&mut self) {
        KEYCHAIN_LOOKUP_OVERRIDE.with(|slot| *slot.borrow_mut() = None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_store(service: &str, account: &str) -> Option<String> {
        match (service, account) {
            ("grok", "new-api") => Some("from-keychain".into()),
            ("grok", "blank") => Some("   ".into()),
            _ => None,
        }
    }

    #[test]
    fn from_parts_account_only_uses_default_service() {
        assert_eq!(
            KeychainRef::from_parts(None, Some("acct")),
            Some(KeychainRef {
                service: DEFAULT_KEYCHAIN_SERVICE.into(),
                account: "acct".into(),
            })
        );
        assert_eq!(
            KeychainRef::from_parts(Some("  "), Some("acct")),
            Some(KeychainRef {
                service: DEFAULT_KEYCHAIN_SERVICE.into(),
                account: "acct".into(),
            })
        );
        assert!(KeychainRef::from_parts(Some("svc"), None).is_none());
        assert!(KeychainRef::from_parts(Some("svc"), Some("  ")).is_none());
        assert!(KeychainRef::from_parts(None, None).is_none());
    }

    #[test]
    fn from_parts_explicit_service_wins() {
        assert_eq!(
            KeychainRef::from_parts(Some(" custom "), Some(" new-api ")),
            Some(KeychainRef {
                service: "custom".into(),
                account: "new-api".into(),
            })
        );
    }

    #[test]
    fn resolve_value_with_fake_backend() {
        let item = KeychainRef {
            service: "grok".into(),
            account: "new-api".into(),
        };
        assert_eq!(
            item.resolve_value_with(fake_store).as_deref(),
            Some("from-keychain")
        );
        let missing = KeychainRef {
            service: "grok".into(),
            account: "missing".into(),
        };
        assert_eq!(missing.resolve_value_with(fake_store), None);
        let blank = KeychainRef {
            service: "grok".into(),
            account: "blank".into(),
        };
        assert_eq!(blank.resolve_value_with(fake_store), None);
    }

    #[test]
    fn platform_missing_item_is_none() {
        assert_eq!(
            platform_lookup_secret(
                "grok-build-keychain-unit-test",
                "definitely-missing-account"
            ),
            None
        );
    }

    #[test]
    fn override_guard_restores_default() {
        let _guard = override_keychain_lookup(fake_store);
        assert_eq!(
            lookup_secret("grok", "new-api").as_deref(),
            Some("from-keychain")
        );
        drop(_guard);
        assert_eq!(
            lookup_secret(
                "grok-build-keychain-unit-test",
                "definitely-missing-account"
            ),
            None
        );
    }
}
