use std::{
    collections::{BTreeMap, BTreeSet},
    vec,
};

use bluer::{
    adv::{Advertisement, AdvertisementHandle},
    gatt::local::{
        characteristic_control, service_control, Application, Characteristic, CharacteristicNotify,
        CharacteristicNotifyMethod, CharacteristicWrite, CharacteristicWriteMethod, Service,
    },
    Adapter, Error,
};
use futures::StreamExt;
use uuid::Uuid;

use crate::gatt::service;

#[derive(Debug)]
pub struct Peripheral {
    adapter: Adapter,
    application: Application,
}

impl Peripheral {
    pub async fn new() -> Result<Self, Error> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;
        println!(
            "Initialize Bluetooth adapter {} with address {}",
            adapter.name(),
            adapter.address().await?
        );
        let application = Application {
            ..Default::default()
        };

        Ok(Peripheral {
            adapter,
            application,
        })
    }

    pub async fn register_gatt(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn unregister_gatt(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn is_powered(&self) -> Result<bool, Error> {
        let result = self.adapter.is_powered().await?;
        return Ok(result);
    }

    pub async fn is_advertising(&self) -> Result<bool, Error> {
        let result = self.adapter.active_advertising_instances().await?;
        return Ok(result > 0);
    }

    pub async fn start_advertising(self: &Self, name: &str, uuids: &[Uuid]) -> Result<(), Error> {
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
        let app_handle = self
            .adapter
            .serve_gatt_application(self.application)
            .await?;
        println!("AdvHandle: {:?}", app_handle);
        Ok(())
    }

    pub async fn stop_advertising(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn add_service(&self, service: &service::Service) -> Result<(), Error> {
        let (_, service_handle) = service_control();

        let mut chars: Vec<Characteristic> = vec![];

        for char in service.characteristics.iter() {
            let (mut char_control, char_handle) = characteristic_control();

            chars.append(&mut vec![Characteristic {
                uuid: char.uuid,
                write: Some(CharacteristicWrite {
                    write: true,
                    write_without_response: true,
                    method: CharacteristicWriteMethod::Io,
                    ..Default::default()
                }),
                notify: Some(CharacteristicNotify {
                    notify: true,
                    method: CharacteristicNotifyMethod::Io,
                    ..Default::default()
                }),
                control_handle: char_handle,
                ..Default::default()
            }]);

            tokio::spawn(async move {
                loop {
                    if let Some(event) = char_control.next().await {
                        print!("event {:?}", event)
                    }
                }
            });
        }

        let service = Service {
            uuid: service.uuid,
            primary: true,
            characteristics: chars,
            control_handle: service_handle,
            ..Default::default()
        };

        self.application.services.append(&mut vec![service]);

        Ok(())
    }
}
