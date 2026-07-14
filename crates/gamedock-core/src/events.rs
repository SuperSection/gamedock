use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    RuntimeStatusChanged {
        runtime_id: String,
        status: String,
    },
    AppInstalled {
        app_id: String,
        package_name: String,
    },
    AppUninstalled {
        app_id: String,
    },
    AppLaunched {
        app_id: String,
    },
    AppClosed {
        app_id: String,
        play_time_seconds: u64,
    },
    UpdateAvailable {
        app_id: String,
        current_version: String,
        new_version: String,
    },
    BackupCreated {
        backup_id: String,
        path: String,
    },
    BackupRestored {
        backup_id: String,
    },
    ControllerConnected {
        controller_id: String,
        controller_type: String,
    },
    ControllerDisconnected {
        controller_id: String,
    },
    Error {
        message: String,
        component: String,
    },
    Progress {
        operation: String,
        current: u64,
        total: u64,
    },
}

#[derive(Debug, Clone)]
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_creation() {
        let _bus = EventBus::new(10);
    }

    #[test]
    fn test_event_bus_default() {
        let _bus = EventBus::default();
    }

    #[test]
    fn test_event_publish_subscribe() {
        let bus = EventBus::new(10);
        let mut rx = bus.subscribe();

        bus.publish(Event::AppInstalled {
            app_id: "test".into(),
            package_name: "com.test".into(),
        });

        let received = rx.try_recv();
        assert!(received.is_ok());
        match received.unwrap() {
            Event::AppInstalled { app_id, package_name } => {
                assert_eq!(app_id, "test");
                assert_eq!(package_name, "com.test");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_event_serialization() {
        let events = vec![
            Event::RuntimeStatusChanged {
                runtime_id: "waydroid".into(),
                status: "Running".into(),
            },
            Event::AppInstalled {
                app_id: "test".into(),
                package_name: "com.test".into(),
            },
            Event::AppUninstalled {
                app_id: "test".into(),
            },
            Event::AppLaunched {
                app_id: "test".into(),
            },
            Event::Error {
                message: "test error".into(),
                component: "test".into(),
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let deserialized: Event = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", event), format!("{:?}", deserialized));
        }
    }
}
