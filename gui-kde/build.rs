use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("com.oppzippy.OpenSCQ30").qml_file("qml/Main.qml"))
        .qt_module("Qml")
        .files([
            "src/bridge/app_controller.rs",
            "src/bridge/connected_device_controller.rs",
            "src/bridge/discovery_controller.rs",
            "src/bridge/quick_presets_controller.rs",
            "src/bridge/session_controller.rs",
        ])
        .build();
}
