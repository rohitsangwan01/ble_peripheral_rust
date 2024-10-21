use std::time::Duration;
use tokio::sync::mpsc::channel;
use uuid::Uuid;

use ble_peripheral_rust::{
    gatt::{
        characteristic::Characteristic,
        descriptor::Descriptor,
        peripheral_event::PeripheralEvent,
        properties::{AttributePermission, CharacteristicProperty},
        service::Service,
    },
    Peripheral, SdpShortUuid,
};

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    if let Err(err) = pretty_env_logger::try_init() {
        eprintln!("WARNING: failed to initialize logging framework: {}", err);
    }

    // Define Characteristics
    let characteristics: Vec<Characteristic> = vec![
        // Char 2A3D
        Characteristic {
            uuid: Uuid::from_sdp_short_uuid(0x2A3D as u16),
            properties: vec![
                CharacteristicProperty::Read,
                CharacteristicProperty::Write,
                CharacteristicProperty::Notify,
            ],
            permissions: vec![
                AttributePermission::Readable,
                AttributePermission::Writeable,
            ],
            value: None,
            descriptors: vec![Descriptor {
                uuid: Uuid::from_sdp_short_uuid(0x2A13 as u16),
                ..Default::default()
            }],
        },
        // Char 1209
        Characteristic {
            uuid: Uuid::from_sdp_short_uuid(0x1209 as u16),
            ..Default::default()
        },
    ];

    // Define Service
    let service = Service {
        uuid: Uuid::from_sdp_short_uuid(0x1234_u16),
        primary: true,
        characteristics: characteristics.clone(),
    };

    let (sender_tx, mut receiver_rx) = channel::<PeripheralEvent>(1);

    let mut peripheral = Peripheral::new(sender_tx).await.unwrap();

    tokio::spawn(async move {
        while let Some(event) = receiver_rx.recv().await {
            log::debug!("Peripheral event: {:?}", event);
            handle_updates(event);
        }
    });

    while !peripheral.is_powered().await.unwrap() {}
    log::info!("Peripheral powered on");

    peripheral.add_service(&service).await.unwrap();

    peripheral
        .start_advertising("RustBLE", &[service.uuid])
        .await
        .unwrap();

    log::info!("Peripheral started advertising");
    let ad_check = async { while !peripheral.is_advertising().await.unwrap() {} };
    let timeout = tokio::time::sleep(Duration::from_secs(60));
    futures::join!(ad_check, timeout);

    peripheral.stop_advertising().await.unwrap();

    while peripheral.is_advertising().await.unwrap() {}
    log::info!("Peripheral stopped advertising");
}

pub fn handle_updates(update: PeripheralEvent) {
    match update {
        PeripheralEvent::DidUpdateState { is_powered } => {
            log::info!("PowerOn: {:?}", is_powered)
        }
        PeripheralEvent::DidStartAdvertising { error } => {
            log::info!("DidStartAdvertising: {:?}", error)
        }
        PeripheralEvent::DidAddService { service, error } => {
            log::info!("DidAddService: {:?} {:?}", service, error)
        }
        PeripheralEvent::DidSubscribeToCharacteristic {
            client,
            service,
            characteristic,
        } => {
            log::info!(
                "DidSubscribeToCharacteristic: {:?} {:?} {:?}",
                client,
                service,
                characteristic
            )
        }
        PeripheralEvent::DidUnsubscribeFromCharacteristic {
            client,
            service,
            characteristic,
        } => {
            log::info!(
                "DidUnsubscribeFromCharacteristic: {:?} {:?} {:?}",
                client,
                service,
                characteristic
            )
        }
        PeripheralEvent::DidReceiveReadRequest {
            client,
            service,
            characteristic,
            responder,
        } => {
            log::info!(
                "DidReceiveReadRequest: {:?} {:?} {:?}",
                client,
                service,
                characteristic
            );
            if let Err(err) = responder.send(String::from("hi").into()) {
                log::error!("Error sending response: {:?}", err);
            }
        }
        PeripheralEvent::DidReceiveWriteRequest {
            client,
            service,
            characteristic,
            value,
        } => {
            log::info!(
                "DidReceiveWriteRequest: {:?} {:?} {:?} {:?}",
                client,
                service,
                characteristic,
                value
            )
        }
    }
}
