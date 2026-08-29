import QtQuick
import org.kde.kirigami as Kirigami
import com.oppzippy.OpenSCQ30

Kirigami.ApplicationWindow {
    id: root

    title: qsTr("OpenSCQ30")

    AppController {
        id: appController
    }

    SessionController {
        id: sessionController

        Component.onCompleted: refreshPairedDevices()
    }

    pageStack.initialPage: Kirigami.Page {
        title: root.title

        ListView {
            anchors.fill: parent
            anchors.margins: Kirigami.Units.largeSpacing
            model: JSON.parse(sessionController.pairedDevicesJson)
            spacing: Kirigami.Units.smallSpacing

            delegate: Item {
                id: deviceItem

                required property var modelData

                height: Kirigami.Units.gridUnit * 3
                width: ListView.view.width

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: Kirigami.Units.smallSpacing

                    Kirigami.Heading {
                        level: 3
                        text: deviceItem.modelData.model
                    }

                    Text {
                        text: deviceItem.modelData.isDemo
                            ? qsTr("Demo device")
                            : deviceItem.modelData.macAddress
                    }
                }
            }

            Kirigami.PlaceholderMessage {
                anchors.centerIn: parent
                text: sessionController.loading
                    ? qsTr("Loading paired devices...")
                    : qsTr("No paired devices")
                visible: parent.count === 0
            }
        }
    }
}
