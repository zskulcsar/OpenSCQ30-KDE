use std::{pin::Pin, str::FromStr, sync::Arc};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use openscq30_lib::connection::ConnectionStatus;

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
        #[qproperty(QString, connection_state, cxx_name = "connectionState")]
        #[qproperty(QString, connected_device_json, cxx_name = "connectedDeviceJson")]
        #[qproperty(QString, error_message, cxx_name = "errorMessage")]
        type ConnectedDeviceController = super::ConnectedDeviceControllerRust;
        #[qinvokable]
        #[cxx_name = "connectDevice"]
        fn connect_device(self: Pin<&mut Self>, mac_address: &QString);
        #[qinvokable]
        #[cxx_name = "cancelConnection"]
        fn cancel_connection(self: Pin<&mut Self>);
        #[qinvokable]
        #[cxx_name = "disconnectDevice"]
        fn disconnect_device(self: Pin<&mut Self>);
    }
    impl cxx_qt::Threading for ConnectedDeviceController {}
}

pub struct ConnectedDeviceControllerRust {
    connection_state: QString,
    connected_device_json: QString,
    error_message: QString,
    services: Arc<AppServices>,
    operations: Arc<OperationLifecycle>,
}

impl Default for ConnectedDeviceControllerRust {
    fn default() -> Self {
        Self {
            connection_state: QString::from("disconnected"),
            connected_device_json: QString::default(),
            error_message: QString::default(),
            services: app_services(),
            operations: Arc::new(OperationLifecycle::default()),
        }
    }
}

impl Drop for ConnectedDeviceControllerRust {
    fn drop(&mut self) {
        self.operations.cancel();
        *self
            .services
            .connected_device
            .lock()
            .expect("connected device lock poisoned") = None;
    }
}

impl ffi::ConnectedDeviceController {
    pub fn connect_device(mut self: Pin<&mut Self>, mac_address: &QString) {
        let mac_address = match macaddr::MacAddr6::from_str(&mac_address.to_string()) {
            Ok(value) => value,
            Err(error) => {
                self.as_mut()
                    .set_error_message(QString::from(&error.to_string()));
                return;
            }
        };
        let operation = self.rust().operations.begin();
        self.as_mut()
            .set_connection_state(QString::from("connecting"));
        self.as_mut().set_error_message(QString::default());
        let thread = self.qt_thread();
        let services = self.rust().services.clone();
        let runtime = services.runtime.clone();
        let monitor_runtime = runtime.clone();
        let lifecycle = self.rust().operations.clone();
        runtime.spawn(async move {
            let result = tokio::select! { () = operation.cancellation.cancelled() => return, result = services.connect(mac_address) => result };
            let _ = thread.queue(move |mut controller| {
                if operation.cancellation.is_cancelled() || !lifecycle.is_current(operation.generation) { return; }
                match result {
                    Ok(device) => {
                        let model = device.model().to_string();
                        *services.connected_device.lock().expect("connected device lock poisoned") = Some(device.clone());
                        controller.as_mut().set_connected_device_json(QString::from(&format!(r#"{{"macAddress":"{mac_address}","model":"{model}"}}"#)));
                        controller.as_mut().set_connection_state(QString::from("connected"));
                        let mut status = device.connection_status(); let thread = controller.qt_thread(); let lifecycle = lifecycle.clone(); let services = services.clone();
                        monitor_runtime.spawn(async move {
                            while status.changed().await.is_ok() {
                                if operation.cancellation.is_cancelled() || !lifecycle.is_current(operation.generation) { return; }
                                if *status.borrow() == ConnectionStatus::Disconnected { *services.connected_device.lock().expect("connected device lock poisoned") = None; let _ = thread.queue(|mut controller| { controller.as_mut().set_connection_state(QString::from("disconnected")); controller.as_mut().set_connected_device_json(QString::default()); }); return; }
                            }
                        });
                    }
                    Err(error) => { controller.as_mut().set_connection_state(QString::from("disconnected")); controller.as_mut().set_error_message(QString::from(&error)); }
                }
            });
        });
    }
    pub fn cancel_connection(mut self: Pin<&mut Self>) {
        self.rust().operations.cancel();
        self.as_mut()
            .set_connection_state(QString::from("disconnected"));
    }
    pub fn disconnect_device(mut self: Pin<&mut Self>) {
        self.rust().operations.cancel();
        *self
            .rust()
            .services
            .connected_device
            .lock()
            .expect("connected device lock poisoned") = None;
        self.as_mut().set_connected_device_json(QString::default());
        self.as_mut()
            .set_connection_state(QString::from("disconnected"));
    }
}
