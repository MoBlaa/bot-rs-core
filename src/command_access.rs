use regex::Regex;

use crate::Message;
use core::fmt;
use serde::export::Formatter;
use std::fmt::Display;

// TODO: Include in derive implementation of StreameblePlugin

/// Manages filters for command invocations. Should be used to manage the access of users
/// to commands based on the messages.
///
/// # Example
///
/// Its recommended to use this before processing the command.
///
/// ```rust,no_run
/// use bot_rs_core::plugin::{Plugin, PluginInfo, PluginError};
/// use bot_rs_core::Message;
/// use async_trait::async_trait;
/// use bot_rs_core::profile::{Profiles, Profile};
///
/// // Profile can be stored on Plugin creation as active profile doesn't change at runtime
/// struct TestPlugin(Profile);
///
/// #[async_trait]
/// impl Plugin for TestPlugin{
///     type Error = PluginError;
///
///     async fn call(&self, message: Message) -> Result<Vec<Message>, PluginError> {
///         match self.0.rights().allowed(&message) {
///             Some(true) => (),// Message was checked by a AccessFilter and is allowed,
///             Some(false) => (),// Message was checked by a AccessFilter and is not allowed
///             None => (), // Message has no handling AccessFilters
///         }
///         Ok(Vec::with_capacity(0))
///     }
///
///     fn info(&self) -> PluginInfo {
///         unimplemented!()
///     }
/// }
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccessRights {
    // Maps the name of an access filter to the filter.
    filters: Vec<AccessFilter>,
}

impl AccessRights {
    /// Creates default access right which allows messages of the broadcaster and non-command invokations.
    pub fn new() -> Self {
        AccessRights {
            filters: vec![AccessFilter::Any(vec![
                AccessFilter::broadcaster(),
                AccessFilter::default_command_start(),
            ])],
        }
    }

    pub const fn empty() -> Self {
        AccessRights {
            filters: Vec::new()
        }
    }

    /// Returns if there are no filters.
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Returns an Iterator over all
    pub fn iter(&self) -> impl Iterator<Item=&AccessFilter> {
        self.filters.iter()
    }

    /// Checks if any filter allows the invocation of a command.
    ///
    /// Returns:
    ///
    /// - `Some(true)` if filters were present for the message and any allowed it,
    /// - `Some(false)` if filters were present for the message and none allowed it and
    /// - `None` if no filters were present for the message.
    pub fn allowed(&self, mssg: &Message) -> Option<bool> {
        let handling = self
            .filters
            .iter()
            .filter(|filter| filter.handles(mssg))
            .collect::<Vec<_>>();
        if handling.is_empty() {
            None
        } else {
            Some(handling.iter().any(|filter| filter.matches(mssg)))
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum AccessFilter {
    /// Checks if a badge matches the given regex.
    Badge(String),
    /// Checks if the user message (IRC = Trailing parameter) matches the given regex string.
    Trailing(String),
    /// Checks if all [AccessFilter]s match a given [Message]. Equivalent to logical `AND`.
    All(Vec<AccessFilter>),
    /// Checks if any [AccessFilter] matches a given [Message]. Equivalent to logical `OR`.
    Any(Vec<AccessFilter>),
}

impl AccessFilter {
    pub fn broadcaster() -> AccessFilter {
        AccessFilter::Badge("broadcaster/*".to_string())
    }

    /// Only allows messages not invoking any commands.
    pub fn default_command_start() -> Self {
        AccessFilter::Trailing(r"^\s*[^!\?¡¿]".to_string())
    }

    /// Returns if the filter is handling the message. This can mean multiple things based
    /// on the type of filter and message:
    ///
    /// - [AccessFilter::Badge] and [Message::Irc] : If the Irc-Message has tags and
    pub fn handles(&self, mssg: &Message) -> bool {
        match (self, mssg) {
            (AccessFilter::Badge(_), Message::Irc(mssg)) => mssg
                .tags()
                .ok()
                .unwrap_or(None)
                .map(|tags| tags.get("badges").is_some())
                .unwrap_or(false),
            (AccessFilter::Trailing(_), Message::Irc(mssg)) => {
                mssg.params().and_then(|params| params.trailing()).is_some()
            }
            (AccessFilter::All(filters), mssg) => filters.iter().all(|filter| filter.handles(mssg)),
            (AccessFilter::Any(filters), mssg) => filters.iter().any(|filter| filter.handles(mssg)),
        }
    }

    pub fn matches(&self, mssg: &Message) -> bool {
        match (self, mssg) {
            (AccessFilter::Badge(regex), Message::Irc(mssg)) => {
                let tags = mssg.tags();
                if tags.is_err() {
                    error!(
                        "failed to parse tags from irc message: {:?}",
                        tags.unwrap_err()
                    );
                    return false;
                }

                tags.unwrap()
                    .and_then(|tags| tags.get("badges"))
                    .map(|badges| {
                        badges.split(',').any(|badge| {
                            let regex = Regex::new(regex).expect("invalid regex");
                            regex.is_match(badge)
                        })
                    })
                    .unwrap_or(false)
            }
            (AccessFilter::Trailing(regex), Message::Irc(mssg)) => mssg
                .params()
                .and_then(|params| params.trailing())
                .map(|trailing| {
                    Regex::new(regex)
                        .expect("invalid trailing regex")
                        .is_match(trailing)
                })
                .unwrap_or(false),
            (AccessFilter::All(filters), mssg) => filters.iter().all(|filter| filter.matches(mssg)),
            (AccessFilter::Any(filters), mssg) => filters.iter().any(|filter| filter.matches(mssg)),
        }
    }
}

impl Display for AccessFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AccessFilter::Badge(regex) => write!(f, "'{}' Badge", regex),
            AccessFilter::Trailing(regex) => write!(f, "'{}' Trailing", regex),
            AccessFilter::All(filters) => {
                write!(f, "( ")?;
                let mut iter = filters.iter();
                if let Some(first) = iter.next() {
                    first.fmt(f)?;
                }
                for filter in iter {
                    write!(f, " AND ")?;
                    filter.fmt(f)?;
                }
                write!(f, " )")
            }
            AccessFilter::Any(filters) => {
                write!(f, "( ")?;
                let mut iter = filters.iter();
                if let Some(first) = iter.next() {
                    first.fmt(f)?;
                }
                for filter in iter {
                    write!(f, " OR ")?;
                    filter.fmt(f)?;
                }
                write!(f, " )")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::command_access::AccessFilter;
    use crate::Message;

    #[test]
    fn test_badge_filter() {
        let badge_filter = AccessFilter::Badge("moderator/*".to_string());

        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .tag("badges", "moderator/1")
                .build(),
        );
        assert!(badge_filter.handles(&message));
        assert!(badge_filter.matches(&message));

        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .tag("badges", "subscriber/1")
                .build(),
        );
        assert!(badge_filter.handles(&message));
        assert!(!badge_filter.matches(&message));

        let message = Message::Irc(irc_rust::Message::builder("PRIVMSG").build());
        assert!(!badge_filter.handles(&message));
        assert!(!badge_filter.matches(&message));
    }

    #[test]
    fn test_trailing_filter() {
        let trailing_filter = AccessFilter::Trailing("^!command$".to_string());

        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("!command")
                .build()
        );
        assert!(trailing_filter.handles(&message));
        assert!(trailing_filter.matches(&message));

        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("Invalid")
                .build()
        );
        assert!(trailing_filter.handles(&message));
        assert!(!trailing_filter.matches(&message));

        let message = Message::Irc(irc_rust::Message::builder("PRIVMSG").build());
        assert!(!trailing_filter.handles(&message));
    }

    #[test]
    fn test_all_filter() {
        let all_filter = AccessFilter::All(vec![
            AccessFilter::Badge("moderator/*".to_string()),
            AccessFilter::Trailing("^hello, world!$".to_string())
        ]);

        // Everything as expected
        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("hello, world!")
                .tag("badges", "moderator/1")
                .build()
        );
        assert!(all_filter.handles(&message));
        assert!(all_filter.matches(&message));

        // Handles but badge doesn't match
        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("hello, world!")
                .tag("badges", "broadcaster/1")
                .build()
        );
        assert!(all_filter.handles(&message));
        assert!(!all_filter.matches(&message));

        // Handles but trailing doesn't match
        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("hello, kevin!")
                .tag("badges", "moderator/1")
                .build()
        );
        assert!(all_filter.handles(&message));
        assert!(!all_filter.matches(&message));

        // Missing tag isn't handled
        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("hello, world!")
                .build()
        );
        assert!(!all_filter.handles(&message));

        // Missing trailing isn't handled
        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .tag("badges", "moderator/1")
                .build()
        );
        assert!(!all_filter.handles(&message));

        let message = Message::Irc(irc_rust::Message::builder("PRIVMSG").build());
        assert!(!all_filter.handles(&message));
    }

    #[test]
    fn test_any_filter() {
        let any_filter = AccessFilter::Any(vec![
            AccessFilter::Badge("moderator/*".to_string()),
            AccessFilter::Trailing("^hello, world!$".to_string())
        ]);

        // Everything as expected
        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("hello, world!")
                .tag("badges", "moderator/1")
                .build()
        );
        assert!(any_filter.handles(&message));
        assert!(any_filter.matches(&message));

        // Matches if any of the filters matches
        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("hello, world!")
                .tag("badges", "broadcaster/1")
                .build()
        );
        assert!(any_filter.handles(&message));
        assert!(any_filter.matches(&message));

        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("hello, kevin!")
                .tag("badges", "moderator/1")
                .build()
        );
        assert!(any_filter.handles(&message));
        assert!(any_filter.matches(&message));

        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .trailing("hello, world!")
                .build()
        );
        assert!(any_filter.handles(&message));
        assert!(any_filter.matches(&message));

        let message = Message::Irc(
            irc_rust::Message::builder("PRIVMSG")
                .tag("badges", "moderator/1")
                .build()
        );
        assert!(any_filter.handles(&message));
        assert!(any_filter.matches(&message));

        let message = Message::Irc(irc_rust::Message::builder("PRIVMSG").build());
        assert!(!any_filter.handles(&message));
    }
}
