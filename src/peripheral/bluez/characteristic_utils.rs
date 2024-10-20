use crate::gatt::characteristic::{self, Secure, Write};
use crate::gatt::peripheral_event::PeripheralEvent;
use crate::gatt::{descriptor, service};
use bluer::gatt::local::{
    service_control, Characteristic, CharacteristicNotifier, CharacteristicNotify,
    CharacteristicNotifyMethod, CharacteristicWrite, CharacteristicWriteMethod,
    CharacteristicWriteRequest, Descriptor, ReqError, Service,
};
use bluer::gatt::local::{CharacteristicRead, CharacteristicReadRequest};
use futures::FutureExt;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

pub fn parse_services(
    gatt_services: Vec<service::Service>,
    sender_tx: Sender<PeripheralEvent>,
) -> Vec<Service> {
    let mut services: Vec<Service> = vec![];

    for service in gatt_services.iter() {
        let (_, service_handle) = service_control();

        let chars: Vec<Characteristic> = service
            .characteristics
            .iter()
            .map(|data| parse_characteristic(data.clone(), service.uuid, sender_tx.clone()))
            .collect();

        let service = Service {
            uuid: service.uuid,
            primary: true,
            characteristics: chars,
            control_handle: service_handle,
            ..Default::default()
        };

        services.push(service);
    }
    services
}

fn parse_characteristic(
    characteristic: characteristic::Characteristic,
    service_uuid: Uuid,
    sender_tx: Sender<PeripheralEvent>,
) -> Characteristic {
    let mut char_read: Option<CharacteristicRead> = None;
    let mut char_write: Option<CharacteristicWrite> = None;
    let mut char_notify: Option<CharacteristicNotify> = None;

    let read_sender = sender_tx.clone();
    if let Some(secure) = &characteristic.properties.read {
        let is_secure = match secure.0 {
            Secure::Secure(_) => true,
            Secure::Insecure(_) => false,
        };
        char_read = Some(CharacteristicRead {
            read: true,
            secure_read: is_secure,
            fun: Box::new(move |request: CharacteristicReadRequest| {
                let sender_tx_clone = read_sender.clone();
                async move {
                    return on_read_request(
                        sender_tx_clone,
                        request,
                        service_uuid,
                        characteristic.uuid,
                    )
                    .await;
                }
                .boxed()
            }),
            ..Default::default()
        })
    }

    let write_sender = sender_tx.clone();
    if let Some(write) = &characteristic.properties.write {
        let mut is_secure = false;
        let with_response = match write {
            Write::WithResponse(secure) => {
                is_secure = match secure {
                    Secure::Secure(_) => true,
                    Secure::Insecure(_) => false,
                };
                true
            }
            Write::WithoutResponse(_) => false,
        };
        char_write = Some(CharacteristicWrite {
            write: !with_response,
            write_without_response: with_response,
            secure_write: is_secure,
            method: CharacteristicWriteMethod::Fun(Box::new(
                move |value: Vec<u8>, request: CharacteristicWriteRequest| {
                    let sender_tx_clone = write_sender.clone();
                    async move {
                        return on_write_request(
                            sender_tx_clone,
                            request,
                            service_uuid,
                            characteristic.uuid,
                            value,
                        )
                        .await;
                    }
                    .boxed()
                },
            )),
            ..Default::default()
        });
    }

    let notify_sender = sender_tx.clone();
    if characteristic.properties.notify.is_some() {
        char_notify = Some(CharacteristicNotify {
            notify: true,
            method: CharacteristicNotifyMethod::Fun(Box::new(
                move |notifier: CharacteristicNotifier| {
                    let sender_tx_clone = notify_sender.clone();
                    async move {
                        return on_char_notify(
                            sender_tx_clone,
                            notifier,
                            service_uuid,
                            characteristic.uuid,
                        )
                        .await;
                    }
                    .boxed()
                },
            )),
            ..Default::default()
        });
    }

    if characteristic.properties.indicate.is_some() {
        char_notify = Some(CharacteristicNotify {
            indicate: true,
            method: CharacteristicNotifyMethod::Io,
            ..Default::default()
        });
    }

    let descriptors: Vec<Descriptor> = characteristic
        .descriptors
        .iter()
        .map(|data| parse_descriptor(data.clone()))
        .collect();

    return Characteristic {
        uuid: characteristic.uuid,
        read: char_read,
        write: char_write,
        notify: char_notify,
        descriptors,
        ..Default::default()
    };
}

fn parse_descriptor(descriptor: descriptor::Descriptor) -> Descriptor {
    // TODO: Add properties
    return Descriptor {
        uuid: descriptor.uuid,
        ..Default::default()
    };
}

/// Handle Requests
async fn on_read_request(
    sender_tx: Sender<PeripheralEvent>,
    request: CharacteristicReadRequest,
    service_uuid: Uuid,
    characteristic: Uuid,
) -> Result<Vec<u8>, ReqError> {
    if let Err(err) = sender_tx
        .send(PeripheralEvent::DidReceiveReadRequest {
            client: request.device_address.to_string(),
            service: service_uuid,
            characteristic: characteristic,
        })
        .await
    {
        eprintln!("Error sending read request event: {:?}", err);
    }
    Ok(vec![])
}

async fn on_write_request(
    sender_tx: Sender<PeripheralEvent>,
    request: CharacteristicWriteRequest,
    service_uuid: Uuid,
    characteristic: Uuid,
    value: Vec<u8>,
) -> Result<(), ReqError> {
    if let Err(err) = sender_tx
        .send(PeripheralEvent::DidReceiveWriteRequest {
            client: request.device_address.to_string(),
            service: service_uuid,
            characteristic: characteristic,
            value,
        })
        .await
    {
        eprintln!("Error sending read request event: {:?}", err);
    }
    Ok(())
}

async fn on_char_notify(
    sender_tx: Sender<PeripheralEvent>,
    notifier: CharacteristicNotifier,
    service_uuid: Uuid,
    characteristic: Uuid,
) {
    if let Err(err) = sender_tx
        .send(PeripheralEvent::DidSubscribeToCharacteristic {
            client: "".to_string(), // Find ClientAddress
            service: service_uuid,
            characteristic: characteristic,
        })
        .await
    {
        eprintln!("Error sending read request event: {:?}", err);
    }
    println!("Notify Requested");

    // notifier.notify(vec![1, 2, 4]).await;

    notifier.stopped().await;
    if let Err(err) = sender_tx
        .send(PeripheralEvent::DidUnsubscribeFromCharacteristic {
            client: "".to_string(), // Find ClientAddress
            service: service_uuid,
            characteristic: characteristic,
        })
        .await
    {
        eprintln!("Error sending read request event: {:?}", err);
    }
    println!("Notify Stopped");
}
