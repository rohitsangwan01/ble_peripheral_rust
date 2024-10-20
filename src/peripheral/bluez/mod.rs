mod characteristic_utils;

use crate::gatt::{peripheral_event::PeripheralEvent, service};
use bluer::{
    adv::{Advertisement, AdvertisementHandle},
    gatt::local::{Application, ApplicationHandle},
    Adapter, Error,
};
use characteristic_utils::parse_services;
use std::collections::{BTreeMap, BTreeSet};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

#[derive(Debug)]
pub struct Peripheral {
    adapter: Adapter,
    services: Vec<service::Service>,
    adv_handle: Option<AdvertisementHandle>,
    app_handle: Option<ApplicationHandle>,
    sender_tx: Sender<PeripheralEvent>,
}

impl Peripheral {
    pub async fn new(sender_tx: Sender<PeripheralEvent>) -> Result<Self, Error> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;
        println!(
            "Initialize Bluetooth adapter {} with address {}",
            adapter.name(),
            adapter.address().await?
        );

        Ok(Peripheral {
            adapter,
            services: Vec::new(),
            adv_handle: None,
            app_handle: None,
            sender_tx,
        })
    }

    pub async fn is_powered(&self) -> Result<bool, Error> {
        let result = self.adapter.is_powered().await?;
        return Ok(result);
    }

    pub async fn is_advertising(&self) -> Result<bool, Error> {
        let result = self.adapter.active_advertising_instances().await?;
        return Ok(result > 0);
    }

    pub async fn start_advertising(&mut self, name: &str, uuids: &[Uuid]) -> Result<(), Error> {
        let manufacturer_data = BTreeMap::new();
        // manufacturer_data.insert(MANUFACTURER_ID, vec![0x21, 0x22, 0x23, 0x24]);

        let mut services: BTreeSet<Uuid> = BTreeSet::new();
        for uuid in uuids {
            services.insert(*uuid);
        }

        let le_advertisement = Advertisement {
            service_uuids: services,
            manufacturer_data,
            discoverable: Some(true),
            local_name: Some(name.to_string()),
            ..Default::default()
        };
        let adv_handle: AdvertisementHandle = self.adapter.advertise(le_advertisement).await?;
        println!("AdvHandle: {:?}", adv_handle);

        let application = Application {
            services: parse_services(self.services.clone(), self.sender_tx.clone()),
            ..Default::default()
        };
        let app_handle = self.adapter.serve_gatt_application(application).await?;
        println!("AdvHandle: {:?}", app_handle);
        self.adv_handle = Some(adv_handle);
        self.app_handle = Some(app_handle);
        Ok(())
    }

    pub async fn stop_advertising(&mut self) -> Result<(), Error> {
        self.adv_handle = None;
        self.app_handle = None;
        Ok(())
    }

    pub async fn add_service(&mut self, service: &service::Service) -> Result<(), Error> {
        self.services.push(service.clone());
        Ok(())
    }
}
