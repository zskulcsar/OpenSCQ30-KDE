use cxx_qt_lib::QString;

use super::{AppServices, app_services};

pub const fn frontend_id() -> &'static str {
    "kde"
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
        #[qproperty(QString, frontend)]
        type AppController = super::AppControllerRust;
    }
}

pub struct AppControllerRust {
    frontend: QString,
    _services: std::sync::Arc<AppServices>,
}

impl Default for AppControllerRust {
    fn default() -> Self {
        Self {
            frontend: QString::from(frontend_id()),
            _services: app_services(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppControllerRust;

    #[test]
    fn app_controller_uses_the_shared_tokio_runtime() {
        let controller = AppControllerRust::default();
        assert_eq!(controller._services.runtime.block_on(async { 2 + 2 }), 4);
    }
}
