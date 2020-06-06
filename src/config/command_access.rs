use std::collections::HashMap;

use regex::Regex;

use crate::Message;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AccessRights {
    /// Maps the name of an access filter to the filter.
    filters: HashMap<String, AccessFilter>
}

impl AccessRights {
    pub fn new() -> Self {
        AccessRights {
            filters: HashMap::new()
        }
    }

    pub fn allowed(&self, mssg: &Message) -> Option<bool> {
        match mssg {
            Message::Irc(irc_mssg) => {
                irc_mssg.params()
                    .and_then(|params| params.trailing)
                    .map(|trailing| {
                        let trailing = trailing.trim_start();
                        for (name, filter) in self.filters.iter() {
                            if trailing.starts_with(name) {
                                return filter.matches(mssg);
                            }
                        }
                        false
                    })
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AccessFilter {
    /// Checks if a badge mathes the given regex.
    BadgeFilter(String)
}

impl AccessFilter {
    pub fn matches(&self, mssg: &Message) -> bool {
        match self {
            AccessFilter::BadgeFilter(regex) => {
                match mssg {
                    Message::Irc(mssg) => {
                        mssg.tags()
                            .and_then(|tags| tags.get("badges"))
                            .map(|badges| {
                                badges.split(',')
                                    .any(|badge| {
                                        let regex = Regex::new(regex).expect("invalid regex");
                                        regex.is_match(badge) || badge.starts_with("broadcaster")
                                    })
                            }).unwrap_or(false)
                    }
                }
            }
        }
    }
}
