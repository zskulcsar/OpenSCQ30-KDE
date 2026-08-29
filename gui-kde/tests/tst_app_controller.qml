import QtQuick
import QtTest
import com.oppzippy.OpenSCQ30

TestCase {
    name: "AppController"

    AppController {
        id: appController
    }

    SessionController {
        id: sessionController
    }

    DiscoveryController {
        id: discoveryController
    }

    ConnectedDeviceController {
        id: connectedDeviceController
    }

    QuickPresetsController {
        id: quickPresetsController
    }

    function test_identifies_kde_frontend() {
        compare(appController.frontend, "kde")
    }

    function test_paired_devices_start_as_an_empty_json_list() {
        compare(sessionController.pairedDevicesJson, "[]")
        compare(sessionController.loading, false)
        compare(sessionController.errorMessage, "")
    }

    function test_refresh_finishes_with_a_json_list() {
        sessionController.refreshPairedDevices()
        tryVerify(function() {
            return !sessionController.loading
        })
        verify(Array.isArray(JSON.parse(sessionController.pairedDevicesJson)))
        compare(sessionController.errorMessage, "")
    }

    function test_demo_discovery_finishes_with_a_json_list() {
        discoveryController.discoverDemoDevices("SoundcoreA3028")
        tryVerify(function() {
            return !discoveryController.loading
        })
        verify(Array.isArray(JSON.parse(discoveryController.devicesJson)))
        compare(discoveryController.errorMessage, "")
    }

    function test_controllers_start_disconnected_without_presets() {
        compare(connectedDeviceController.connectionState, "disconnected")
        compare(connectedDeviceController.connectedDeviceJson, "")
        compare(quickPresetsController.presetsJson, "[]")
        compare(quickPresetsController.loading, false)
    }

    function test_demo_device_can_be_paired_connected_and_loaded() {
        discoveryController.discoverDemoDevices("SoundcoreA3028")
        tryVerify(function() {
            return !discoveryController.loading
        })
        var devices = JSON.parse(discoveryController.devicesJson)
        verify(devices.length > 0)

        sessionController.pairDevice(devices[0].macAddress, "SoundcoreA3028", true)
        tryVerify(function() {
            return !sessionController.loading
        })
        compare(sessionController.errorMessage, "")

        connectedDeviceController.connectDevice(devices[0].macAddress)
        tryVerify(function() {
            return connectedDeviceController.connectionState === "connected"
        })
        compare(JSON.parse(connectedDeviceController.connectedDeviceJson).macAddress,
                devices[0].macAddress)

        quickPresetsController.refreshPresets()
        tryVerify(function() {
            return !quickPresetsController.loading
        })
        verify(Array.isArray(JSON.parse(quickPresetsController.presetsJson)))
        compare(quickPresetsController.errorMessage, "")

        connectedDeviceController.disconnectDevice()
        compare(connectedDeviceController.connectionState, "disconnected")

        sessionController.removeDevice(devices[0].macAddress)
        tryVerify(function() {
            return !sessionController.loading
        })
        compare(sessionController.errorMessage, "")
    }
}
