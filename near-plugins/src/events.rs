//! # NEAR Events
//!
//! Description of Events on NEAR following [NEP-297](https://nomicon.io/Standards/EventsFormat)
use serde::Serialize;

/// Interface to capture metadata about an event
#[derive(Serialize)]
pub struct EventMetadata<T> {
    /// name of standard, e.g. nep171
    pub standard: String,
    /// e.g. 1.0.0
    pub version: String,
    /// type of the event, e.g. nft_mint
    pub event: String,
    /// associate event data. Strictly typed for each set {standard, version, event}
    /// inside corresponding NEP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// Trait to generate and emit NEAR events.
pub trait AsEvent<T: Serialize> {
    /// Returns the metadata that makes up the event.
    fn metadata(&self) -> EventMetadata<T>;

    /// Returns the string representation of the event.
    fn event(&self) -> String {
        format!(
            "EVENT_JSON:{}",
            near_sdk::serde_json::to_string(&self.metadata()).unwrap()
        )
    }

    /// Emits the event on chain.
    fn emit(&self) {
        near_sdk::log!("{}", self.event());
    }
}

#[cfg(test)]
mod tests {
    use crate::events::{AsEvent, EventMetadata};

    struct CompileEvent {
        info: Option<String>,
    }

    impl AsEvent<String> for CompileEvent {
        fn metadata(&self) -> EventMetadata<String> {
            EventMetadata {
                standard: "Compile".to_string(),
                version: "0.0.1".to_string(),
                event: "compile_test".to_string(),
                data: self.info.clone(),
            }
        }
    }

    /// Helper function to check if an event is well formed and follows NEP-297
    /// i.e. tries to deserialize the json object.
    fn valid_event(event: &str) -> bool {
        #[derive(serde::Deserialize)]
        struct EventFormat {
            #[allow(dead_code)]
            standard: String,
            #[allow(dead_code)]
            version: String,
            #[allow(dead_code)]
            event: String,
            #[allow(dead_code)]
            data: Option<near_sdk::serde_json::Value>,
        }

        let prefix = "EVENT_JSON:";
        if !event.starts_with(prefix) {
            return false;
        }
        let r = &event[prefix.len()..];
        near_sdk::serde_json::from_str::<EventFormat>(r).is_ok()
    }

    #[test]
    fn event_no_data() {
        let compile_event = CompileEvent { info: None };
        let event_log = compile_event.event();
        let expected =
            r#"EVENT_JSON:{"standard":"Compile","version":"0.0.1","event":"compile_test"}"#;
        assert_eq!(event_log, expected);
        assert!(valid_event(&event_log));
    }

    #[test]
    fn event_with_data() {
        let compile_event = CompileEvent {
            info: Some("Compilation successful".to_string()),
        };
        let event_log = compile_event.event();
        let expected = r#"EVENT_JSON:{"standard":"Compile","version":"0.0.1","event":"compile_test","data":"Compilation successful"}"#;
        assert_eq!(event_log, expected);
        assert!(valid_event(&event_log));
    }
}
