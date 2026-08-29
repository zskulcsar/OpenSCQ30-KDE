use std::{
    future::Future,
    pin::Pin,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use super::{AppServices, OperationLifecycle, app_services};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use openscq30_lib::{DeviceModel, storage::PairedDevice};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedDeviceDto {
    pub mac_address: String,
    pub model: String,
    pub is_demo: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDescriptorDto {
    pub name: String,
    pub mac_address: String,
}

impl ConnectionDescriptorDto {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("connection descriptor DTO must be serializable")
    }
}

impl PairedDeviceDto {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("paired device DTO must be serializable")
    }
}

impl From<PairedDevice> for PairedDeviceDto {
    fn from(device: PairedDevice) -> Self {
        Self {
            mac_address: device.mac_address.to_string(),
            model: device.model.to_string(),
            is_demo: device.is_demo,
        }
    }
}

#[derive(Default)]
pub struct RefreshGeneration(AtomicU64);

impl RefreshGeneration {
    pub fn begin(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.0.load(Ordering::Acquire) == generation
    }
}

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, paired_devices_json, cxx_name = "pairedDevicesJson")]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error_message, cxx_name = "errorMessage")]
        type SessionController = super::SessionControllerRust;

        #[qinvokable]
        #[cxx_name = "refreshPairedDevices"]
        fn refresh_paired_devices(self: Pin<&mut Self>);
        #[qinvokable]
        #[cxx_name = "pairDevice"]
        fn pair_device(self: Pin<&mut Self>, mac_address: &QString, model: &QString, is_demo: bool);
        #[qinvokable]
        #[cxx_name = "removeDevice"]
        fn remove_device(self: Pin<&mut Self>, mac_address: &QString);
    }

    impl cxx_qt::Threading for SessionController {}
}

pub struct SessionControllerRust {
    paired_devices_json: QString,
    loading: bool,
    error_message: QString,
    services: Arc<AppServices>,
    operations: Arc<OperationLifecycle>,
}

impl Default for SessionControllerRust {
    fn default() -> Self {
        Self {
            paired_devices_json: QString::from("[]"),
            loading: false,
            error_message: QString::default(),
            services: app_services(),
            operations: Arc::new(OperationLifecycle::default()),
        }
    }
}

impl Drop for SessionControllerRust {
    fn drop(&mut self) {
        self.operations.cancel();
    }
}

impl ffi::SessionController {
    pub fn refresh_paired_devices(mut self: Pin<&mut Self>) {
        let operation = self.rust().operations.begin();

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::default());

        let qt_thread = self.qt_thread();
        let services = self.rust().services.clone();
        let runtime = services.runtime.clone();
        let operations = self.rust().operations.clone();

        runtime.spawn(async move {
            let result = tokio::select! {
                () = operation.cancellation.cancelled() => return,
                result = services.paired_devices() => result,
            };

            if operation.cancellation.is_cancelled() {
                return;
            }

            let _ = qt_thread.queue(move |mut controller| {
                if operation.cancellation.is_cancelled()
                    || !operations.is_current(operation.generation)
                {
                    return;
                }

                match result {
                    Ok(devices) => {
                        let devices = devices
                            .into_iter()
                            .map(PairedDeviceDto::from)
                            .collect::<Vec<_>>();
                        controller.as_mut().set_paired_devices_json(QString::from(
                            serde_json::to_string(&devices)
                                .expect("paired device DTOs must be serializable"),
                        ));
                        controller.as_mut().set_error_message(QString::default());
                    }
                    Err(error) => controller.as_mut().set_error_message(QString::from(&error)),
                }
                controller.as_mut().set_loading(false);
            });
        });
    }

    pub fn pair_device(
        mut self: Pin<&mut Self>,
        mac_address: &QString,
        model: &QString,
        is_demo: bool,
    ) {
        let device = match (
            macaddr::MacAddr6::from_str(&mac_address.to_string()),
            DeviceModel::from_str(&model.to_string()),
        ) {
            (Ok(mac_address), Ok(model)) => PairedDevice {
                mac_address,
                model,
                is_demo,
            },
            (Err(error), _) => {
                self.as_mut()
                    .set_error_message(QString::from(&error.to_string()));
                return;
            }
            (_, Err(error)) => {
                self.as_mut()
                    .set_error_message(QString::from(&error.to_string()));
                return;
            }
        };
        self.refresh_after(move |services| async move { services.pair(device).await });
    }

    pub fn remove_device(mut self: Pin<&mut Self>, mac_address: &QString) {
        let mac_address = match macaddr::MacAddr6::from_str(&mac_address.to_string()) {
            Ok(value) => value,
            Err(error) => {
                self.as_mut()
                    .set_error_message(QString::from(&error.to_string()));
                return;
            }
        };
        self.refresh_after(move |services| async move { services.unpair(mac_address).await });
    }
}

impl ffi::SessionController {
    fn refresh_after<F, Fut>(mut self: Pin<&mut Self>, operation: F)
    where
        F: FnOnce(Arc<AppServices>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let operation_state = self.rust().operations.begin();
        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::default());
        let thread = self.qt_thread();
        let services = self.rust().services.clone();
        let runtime = services.runtime.clone();
        let lifecycle = self.rust().operations.clone();
        runtime.spawn(async move {
            let result = tokio::select! { () = operation_state.cancellation.cancelled() => return, result = operation(services.clone()) => result };
            let _ = thread.queue(move |mut controller| {
                if operation_state.cancellation.is_cancelled() || !lifecycle.is_current(operation_state.generation) { return; }
                let succeeded = result.is_ok();
                if let Err(error) = result { controller.as_mut().set_error_message(QString::from(&error)); }
                controller.as_mut().set_loading(false);
                if succeeded { controller.as_mut().refresh_paired_devices(); }
            });
        });
    }
}
