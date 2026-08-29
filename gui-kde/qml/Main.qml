import QtQuick
import org.kde.kirigami as Kirigami
import com.oppzippy.OpenSCQ30

Kirigami.ApplicationWindow {
    id: root

    title: qsTr("OpenSCQ30")

    AppController {
        id: appController
    }

    pageStack.initialPage: Kirigami.Page {
        title: root.title

        Kirigami.Heading {
            anchors.centerIn: parent
            level: 1
            text: qsTr("OpenSCQ30 KDE frontend")
        }
    }

}
