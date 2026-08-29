use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

use super::{AppServices, ConnectionDescriptorDto, OperationLifecycle, app_services};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use openscq30_lib::{DeviceModel, connection::ConnectionDescriptor};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, devices_json, cxx_name = "devicesJson")]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error_message, cxx_name = "errorMessage")]
        type DiscoveryController = super::DiscoveryControllerRust;

        #[qinvokable]
        #[cxx_name = "discoverDemoDevices"]
        fn discover_demo_devices(self: Pin<&mut Self>, model: &QString);
        #[qinvokable]
        #[cxx_name = "discoverPhysicalDevices"]
        fn discover_physical_devices(self: Pin<&mut Self>, model: &QString);
    }

    impl cxx_qt::Threading for DiscoveryController {}
}

pub struct DiscoveryControllerRust {
    devices_json: QString,
    loading: bool,
    error_message: QString,
    services: Arc<AppServices>,
    operations: Arc<OperationLifecycle>,
}

impl Default for DiscoveryControllerRust {
    fn default() -> Self {
        Self {
            devices_json: QString::from("[]"),
            loading: false,
            error_message: QString::default(),
            services: app_services(),
            operations: Arc::new(OperationLifecycle::default()),
        }
    }
}

impl Drop for DiscoveryControllerRust {
    fn drop(&mut self) {
        self.operations.cancel();
    }
}

impl ffi::DiscoveryController {
    pub fn discover_demo_devices(mut self: Pin<&mut Self>, model: &QString) {
        let model = match DeviceModel::from_str(&model.to_string()) {
            Ok(model) => model,
            Err(error) => {
                self.as_mut()
                    .set_error_message(QString::from(&error.to_string()));
                return;
            }
        };
        self.discover(model, |services, model| async move {
            services.demo_devices(model).await
        });
    }

    pub fn discover_physical_devices(mut self: Pin<&mut Self>, model: &QString) {
        let model = match DeviceModel::from_str(&model.to_string()) {
            Ok(model) => model,
            Err(error) => {
                self.as_mut()
                    .set_error_message(QString::from(&error.to_string()));
                return;
            }
        };
        self.discover(model, |services, model| async move {
            services.physical_devices(model).await
        });
    }
}

impl ffi::DiscoveryController {
    fn discover<F, Fut>(mut self: Pin<&mut Self>, model: DeviceModel, discover: F)
    where
        F: FnOnce(Arc<AppServices>, DeviceModel) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Vec<ConnectionDescriptor>, String>> + Send + 'static,
    {
        let operation = self.rust().operations.begin();
        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::default());
        let thread = self.qt_thread();
        let services = self.rust().services.clone();
        let runtime = services.runtime.clone();
        let operations = self.rust().operations.clone();
        runtime.spawn(async move {
            let result = tokio::select! { () = operation.cancellation.cancelled() => return, result = discover(services, model) => result };
            let _ = thread.queue(move |mut controller| {
                if operation.cancellation.is_cancelled() || !operations.is_current(operation.generation) { return; }
                match result { Ok(devices) => controller.as_mut().set_devices_json(QString::from(serde_json::to_string(&devices.into_iter().map(ConnectionDescriptorDto::from).collect::<Vec<_>>()).expect("connection DTOs must serialize"))), Err(error) => controller.as_mut().set_error_message(QString::from(&error)) }
                controller.as_mut().set_loading(false);
            });
        });
    }
}

impl From<ConnectionDescriptor> for ConnectionDescriptorDto {
    fn from(device: ConnectionDescriptor) -> Self {
        Self {
            name: device.name,
            mac_address: device.mac_address.to_string(),
        }
    }
}
