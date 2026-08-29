pub mod bridge;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::bridge::{
        ConnectionDescriptorDto, OperationLifecycle, PairedDeviceDto, RefreshGeneration,
        app_services,
    };

    #[test]
    fn app_controller_identifies_the_kde_frontend() {
        assert_eq!(super::bridge::frontend_id(), "kde");
    }

    #[test]
    fn paired_device_dto_uses_the_android_compatible_json_shape() {
        let device = PairedDeviceDto {
            mac_address: "00:11:22:33:44:55".to_owned(),
            model: "SoundcoreA3028".to_owned(),
            is_demo: true,
        };

        assert_eq!(
            device.to_json(),
            r#"{"macAddress":"00:11:22:33:44:55","model":"SoundcoreA3028","isDemo":true}"#
        );
    }

    #[test]
    fn newer_refresh_suppresses_an_older_completion() {
        let refresh = RefreshGeneration::default();
        let older = refresh.begin();
        let newer = refresh.begin();

        assert!(!refresh.is_current(older));
        assert!(refresh.is_current(newer));
    }

    #[test]
    fn new_operation_cancels_and_supersedes_the_previous_operation() {
        let operations = OperationLifecycle::default();
        let first = operations.begin();
        let second = operations.begin();

        assert!(first.cancellation.is_cancelled());
        assert!(!operations.is_current(first.generation));
        assert!(operations.is_current(second.generation));
    }

    #[test]
    fn teardown_suppresses_an_outstanding_refresh_completion() {
        let refresh = RefreshGeneration::default();
        let outstanding = refresh.begin();
        refresh.begin();

        assert!(!refresh.is_current(outstanding));
    }

    #[test]
    fn controllers_share_one_app_service_factory() {
        assert!(Arc::ptr_eq(&app_services(), &app_services()));
    }

    #[test]
    fn connection_descriptor_dto_uses_the_android_compatible_json_shape() {
        let device = ConnectionDescriptorDto {
            name: "Soundcore Q30".to_owned(),
            mac_address: "00:11:22:33:44:55".to_owned(),
        };

        assert_eq!(
            device.to_json(),
            r#"{"name":"Soundcore Q30","macAddress":"00:11:22:33:44:55"}"#
        );
    }
}
