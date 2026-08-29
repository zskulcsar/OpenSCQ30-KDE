use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("com.oppzippy.OpenSCQ30").qml_file("qml/Main.qml"))
        .qt_module("Qml")
        .files(["src/bridge/app_controller.rs"])
        .build();
}
