use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use openscq30_lib::settings::SettingId;

use super::{AppServices, OperationLifecycle, app_services};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, presets_json, cxx_name = "presetsJson")]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error_message, cxx_name = "errorMessage")]
        type QuickPresetsController = super::QuickPresetsControllerRust;
        #[qinvokable]
        #[cxx_name = "refreshPresets"]
        fn refresh_presets(self: Pin<&mut Self>);
        #[qinvokable]
        #[cxx_name = "savePreset"]
        fn save_preset(self: Pin<&mut Self>, name: &QString);
        #[qinvokable]
        #[cxx_name = "deletePreset"]
        fn delete_preset(self: Pin<&mut Self>, name: &QString);
        #[qinvokable]
        #[cxx_name = "applyPreset"]
        fn apply_preset(self: Pin<&mut Self>, name: &QString);
        #[qinvokable]
        #[cxx_name = "togglePresetField"]
        fn toggle_preset_field(
            self: Pin<&mut Self>,
            name: &QString,
            setting_id: &QString,
            enabled: bool,
        );
    }
    impl cxx_qt::Threading for QuickPresetsController {}
}

pub struct QuickPresetsControllerRust {
    presets_json: QString,
    loading: bool,
    error_message: QString,
    services: Arc<AppServices>,
    operations: Arc<OperationLifecycle>,
}
impl Default for QuickPresetsControllerRust {
    fn default() -> Self {
        Self {
            presets_json: QString::from("[]"),
            loading: false,
            error_message: QString::default(),
            services: app_services(),
            operations: Arc::new(OperationLifecycle::default()),
        }
    }
}
impl Drop for QuickPresetsControllerRust {
    fn drop(&mut self) {
        self.operations.cancel();
    }
}

impl ffi::QuickPresetsController {
    pub fn refresh_presets(self: Pin<&mut Self>) {
        self.run(|handler, device| async move {
            handler
                .quick_presets(device.as_ref())
                .await
                .map_err(|error| error.to_string())
                .map(|presets| {
                    Some(serde_json::to_string(&presets).expect("quick presets must serialize"))
                })
        });
    }
    pub fn save_preset(self: Pin<&mut Self>, name: &QString) {
        let name = name.to_string();
        self.run(move |handler, device| async move {
            handler
                .save(device.as_ref(), name)
                .await
                .map_err(|error| error.to_string())
                .map(|_| None)
        });
    }
    pub fn delete_preset(self: Pin<&mut Self>, name: &QString) {
        let name = name.to_string();
        self.run(move |handler, device| async move {
            handler
                .delete(device.as_ref(), name)
                .await
                .map_err(|error| error.to_string())
                .map(|_| None)
        });
    }
    pub fn apply_preset(self: Pin<&mut Self>, name: &QString) {
        let name = name.to_string();
        self.run(move |handler, device| async move {
            handler
                .activate(device.as_ref(), name)
                .await
                .map_err(|error| error.to_string())
                .map(|_| None)
        });
    }
    pub fn toggle_preset_field(
        mut self: Pin<&mut Self>,
        name: &QString,
        setting_id: &QString,
        enabled: bool,
    ) {
        let setting_id = match SettingId::from_str(&setting_id.to_string()) {
            Ok(value) => value,
            Err(error) => {
                self.as_mut()
                    .set_error_message(QString::from(&error.to_string()));
                return;
            }
        };
        let name = name.to_string();
        self.run(move |handler, device| async move {
            handler
                .toggle_field(device.as_ref(), name, setting_id, enabled)
                .await
                .map_err(|error| error.to_string())
                .map(|_| None)
        });
    }
    fn run<F, Fut>(mut self: Pin<&mut Self>, action: F)
    where
        F: FnOnce(
                openscq30_lib::quick_presets::QuickPresetsHandler,
                Arc<dyn openscq30_lib::device::OpenSCQ30Device + Send + Sync>,
            ) -> Fut
            + Send
            + 'static,
        Fut: Future<Output = Result<Option<String>, String>> + Send + 'static,
    {
        let connected_device = self
            .rust()
            .services
            .connected_device
            .lock()
            .expect("connected device lock poisoned")
            .clone();
        let device = match connected_device {
            Some(device) => device,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("No device is connected."));
                return;
            }
        };
        let operation = self.rust().operations.begin();
        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::default());
        let thread = self.qt_thread();
        let services = self.rust().services.clone();
        let runtime = services.runtime.clone();
        let lifecycle = self.rust().operations.clone();
        runtime.spawn(async move {
            let result = tokio::select! { () = operation.cancellation.cancelled() => return, result = async { action(services.quick_preset_handler().await?, device).await } => result };
            let _ = thread.queue(move |mut controller| { if operation.cancellation.is_cancelled() || !lifecycle.is_current(operation.generation) { return; } match result { Ok(Some(presets)) => controller.as_mut().set_presets_json(QString::from(&presets)), Ok(None) => (), Err(error) => controller.as_mut().set_error_message(QString::from(&error)) }; controller.as_mut().set_loading(false); });
        });
    }
}
