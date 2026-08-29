import QtQuick
import QtTest
import com.oppzippy.OpenSCQ30

TestCase {
    name: "AppController"

    AppController {
        id: appController
    }

    function test_identifies_kde_frontend() {
        compare(appController.frontend, "kde")
    }
}
