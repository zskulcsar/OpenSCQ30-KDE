use cxx_qt_lib::QString;

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
    _runtime: tokio::runtime::Runtime,
}

impl Default for AppControllerRust {
    fn default() -> Self {
        Self {
            frontend: QString::from(frontend_id()),
            _runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to create the KDE Tokio runtime"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppControllerRust;

    #[test]
    fn app_controller_starts_a_tokio_runtime() {
        let controller = AppControllerRust::default();
        assert_eq!(controller._runtime.block_on(async { 2 + 2 }), 4);
    }
}
