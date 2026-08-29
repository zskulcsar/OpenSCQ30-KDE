use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use openscq30_lib::{
    OpenSCQ30Session, connection::ConnectionDescriptor, device::OpenSCQ30Device,
    storage::PairedDevice,
};
use tokio::sync::OnceCell;

pub struct AppServices {
    pub runtime: Arc<tokio::runtime::Runtime>,
    database_path: Option<PathBuf>,
    session: OnceCell<Arc<OpenSCQ30Session>>,
    pub connected_device: std::sync::Mutex<Option<Arc<dyn OpenSCQ30Device + Send + Sync>>>,
}

impl AppServices {
    async fn session(&self) -> Result<Arc<OpenSCQ30Session>, String> {
        self.session
            .get_or_try_init(|| async {
                let database_path = self
                    .database_path
                    .clone()
                    .ok_or_else(|| "Unable to determine the configuration directory.".to_owned())?;
                OpenSCQ30Session::new(database_path)
                    .await
                    .map(Arc::new)
                    .map_err(|error| error.to_string())
            })
            .await
            .cloned()
    }

    pub async fn paired_devices(
        &self,
    ) -> Result<Vec<openscq30_lib::storage::PairedDevice>, String> {
        self.session()
            .await?
            .paired_devices()
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn demo_devices(
        &self,
        model: openscq30_lib::DeviceModel,
    ) -> Result<Vec<openscq30_lib::connection::ConnectionDescriptor>, String> {
        self.session()
            .await?
            .list_demo_devices(model)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn physical_devices(
        &self,
        model: openscq30_lib::DeviceModel,
    ) -> Result<Vec<ConnectionDescriptor>, String> {
        self.session()
            .await?
            .list_devices(model)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn pair(&self, device: PairedDevice) -> Result<(), String> {
        self.session()
            .await?
            .pair(device)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn unpair(&self, mac_address: macaddr::MacAddr6) -> Result<(), String> {
        self.session()
            .await?
            .unpair(mac_address)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn connect(
        &self,
        mac_address: macaddr::MacAddr6,
    ) -> Result<Arc<dyn OpenSCQ30Device + Send + Sync>, String> {
        self.session()
            .await?
            .connect(mac_address)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn quick_preset_handler(
        &self,
    ) -> Result<openscq30_lib::quick_presets::QuickPresetsHandler, String> {
        Ok(self.session().await?.quick_preset_handler())
    }
}

pub fn app_services() -> Arc<AppServices> {
    static SERVICES: OnceLock<Arc<AppServices>> = OnceLock::new();

    SERVICES
        .get_or_init(|| {
            Arc::new(AppServices {
                runtime: Arc::new(
                    tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .expect("failed to create the KDE Tokio runtime"),
                ),
                database_path: std::env::var_os("OPENSCQ30_DATABASE_PATH")
                    .map(PathBuf::from)
                    .or_else(|| {
                        dirs::config_dir()
                            .map(|path| path.join("openscq30").join("database.sqlite"))
                    }),
                session: OnceCell::new(),
                connected_device: std::sync::Mutex::new(None),
            })
        })
        .clone()
}
