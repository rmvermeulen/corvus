use std::sync::{Arc, Mutex, mpsc};

use bevy::log::tracing::{Level, Subscriber};
use bevy::log::tracing_subscriber::{Layer, layer};
use bevy::log::{LogPlugin, tracing};
use derive_more::From;

use crate::prelude::*;

#[derive(Debug, Deref, DerefMut, Event, From)]
struct CobwebWarning {
    message: String,
}

struct CobwebWarningLayer(mpsc::Sender<CobwebWarning>);

impl<S: Subscriber> Layer<S> for CobwebWarningLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: layer::Context<'_, S>) {
        let meta = event.metadata();
        if meta.level() == &Level::WARN
            && meta
                .module_path()
                .unwrap_or_else(|| meta.target())
                .contains("bevy_cobweb_ui_core::extract")
            && let Some(field) = event.fields().find(|f| f.name() == "message")
        {
            self.0
                .send(CobwebWarning::from(field.to_string()))
                .expect("Failed to send CobwebWarning");
        }
    }
}

#[derive(Debug, Deref, DerefMut, Resource)]
struct CobwebWarningReceiver(Arc<Mutex<mpsc::Receiver<CobwebWarning>>>);

impl From<mpsc::Receiver<CobwebWarning>> for CobwebWarningReceiver {
    fn from(receiver: mpsc::Receiver<CobwebWarning>) -> Self {
        CobwebWarningReceiver(Arc::new(Mutex::new(receiver)))
    }
}

pub fn get_log_plugin() -> LogPlugin {
    // in debug mode we forward cobweb's warnings to the app
    LogPlugin {
        custom_layer: |app| {
            let (sender, receiver) = mpsc::channel();
            app.add_event::<CobwebWarning>()
                .insert_resource(CobwebWarningReceiver::from(receiver))
                .add_systems(
                    Update,
                    |mut writer: EventWriter<CobwebWarning>, cwr: Res<CobwebWarningReceiver>| {
                        if let Ok(lock) = cwr.try_lock()
                            && let Ok(warning) = lock.try_recv()
                        {
                            writer.write(warning);
                        }
                    },
                );
            Some(CobwebWarningLayer(sender).boxed())
        },
        ..default()
    }
}
