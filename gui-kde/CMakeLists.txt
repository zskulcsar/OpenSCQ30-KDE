cmake_minimum_required(VERSION 3.30)

project(openscq30_gui_kde LANGUAGES CXX)

include(CTest)

set(CMAKE_AUTOMOC ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.8 REQUIRED COMPONENTS Core Gui Qml QmlTools Quick QuickControls2 QuickTest Test)
find_package(KF6Kirigami REQUIRED)

include(FetchContent)
FetchContent_Declare(
    CxxQt
    GIT_REPOSITORY https://github.com/KDAB/cxx-qt-cmake.git
    GIT_TAG 64e63f560737b3b7f85ef8267c5918e4b8176d5a
    GIT_SHALLOW TRUE
)
FetchContent_MakeAvailable(CxxQt)

cxx_qt_import_crate(
    MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml
    CRATES openscq30_gui_kde
    CRATE_TYPES staticlib
    LOCKED
    QT_MODULES Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick Qt6::QuickControls2
)

cxx_qt_import_qml_module(openscq30_gui_kde_qml
    URI "com.oppzippy.OpenSCQ30"
    SOURCE_CRATE openscq30_gui_kde)

add_executable(openscq30-gui cpp/main.cpp)
target_link_libraries(openscq30-gui PRIVATE
    openscq30_gui_kde_qml
    KF6::Kirigami
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2)

if(BUILD_TESTING)
    add_executable(openscq30-gui-qml-test tests/tst_app_controller.cpp)
    target_link_libraries(openscq30-gui-qml-test PRIVATE
        openscq30_gui_kde_qml
        Qt6::QuickTest)
    add_test(NAME app-controller-qml
        COMMAND openscq30-gui-qml-test -input ${CMAKE_CURRENT_SOURCE_DIR}/tests/tst_app_controller.qml)
    set_tests_properties(app-controller-qml PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=offscreen")

    get_target_property(QMLLINT_COMMAND Qt6::qmllint IMPORTED_LOCATION)
    add_test(NAME qmllint
        COMMAND ${QMLLINT_COMMAND} --max-warnings 0
            -I ${CMAKE_CURRENT_BINARY_DIR}/cxxqt/qml_modules
            ${CMAKE_CURRENT_SOURCE_DIR}/qml/Main.qml
            ${CMAKE_CURRENT_SOURCE_DIR}/tests/tst_app_controller.qml)
endif()

install(TARGETS openscq30-gui RUNTIME DESTINATION bin)
install(FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/../gui/resources/com.oppzippy.OpenSCQ30.desktop
    DESTINATION share/applications)
install(FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/../gui/resources/com.oppzippy.OpenSCQ30.metainfo.xml
    DESTINATION share/metainfo)
install(FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/../gui/resources/com.oppzippy.OpenSCQ30.svg
    DESTINATION share/icons/hicolor/scalable/apps)
