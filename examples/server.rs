use futures::{channel::mpsc::channel, prelude::*};
use std::{
    collections::HashSet,
    sync::{atomic, Arc, Mutex},
    thread,
    time::Duration,
};
use uuid::Uuid;

use ble_peripheral_rust::{
    gatt::{
        characteristic::{self, Characteristic},
        descriptor::{self, Descriptor},
        event::{Event, Response},
        peripheral_event::PeripheralEvent,
        service::Service,
    },
    Peripheral, SdpShortUuid,
};

const ADVERTISING_NAME: &str = "hello";
const ADVERTISING_TIMEOUT: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() {
    if let Err(err) = pretty_env_logger::try_init() {
        eprintln!("WARNING: failed to initialize logging framework: {}", err);
    }

    let (sender_characteristic, receiver_characteristic) = channel(1);
    let (sender_descriptor, receiver_descriptor) = channel(1);

    let mut characteristics: HashSet<Characteristic> = HashSet::new();

    characteristics.insert(Characteristic::new(
        Uuid::from_sdp_short_uuid(0x2A3D as u16),
        characteristic::Properties::new(
            Some(characteristic::Read(characteristic::Secure::Insecure(
                sender_characteristic.clone(),
            ))),
            Some(characteristic::Write::WithResponse(
                characteristic::Secure::Insecure(sender_characteristic.clone()),
            )),
            Some(sender_characteristic),
            None,
        ),
        None,
        {
            let mut descriptors = HashSet::<Descriptor>::new();
            descriptors.insert(Descriptor::new(
                Uuid::from_sdp_short_uuid(0x2A3D as u16),
                descriptor::Properties::new(
                    Some(descriptor::Read(descriptor::Secure::Insecure(
                        sender_descriptor.clone(),
                    ))),
                    Some(descriptor::Write(descriptor::Secure::Insecure(
                        sender_descriptor,
                    ))),
                ),
                None,
            ));
            descriptors
        },
    ));

    let characteristic_handler = async {
        let characteristic_value = Arc::new(Mutex::new(String::from("hi")));
        let notifying: Arc<atomic::AtomicBool> = Arc::new(atomic::AtomicBool::new(false));
        let mut rx = receiver_characteristic;
        while let Some(event) = rx.next().await {
            match event {
                Event::ReadRequest(read_request) => {
                    println!(
                        "GATT server got a read request with offset {}!",
                        read_request.offset
                    );
                    let value = characteristic_value.lock().unwrap().clone();
                    read_request
                        .response
                        .send(Response::Success(value.clone().into()))
                        .unwrap();
                    println!("GATT server responded with \"{}\"", value);
                }
                Event::WriteRequest(write_request) => {
                    let new_value = String::from_utf8(write_request.data).unwrap();
                    println!(
                        "GATT server got a write request with offset {} and data {}!",
                        write_request.offset, new_value,
                    );
                    *characteristic_value.lock().unwrap() = new_value;
                    write_request
                        .response
                        .send(Response::Success(vec![]))
                        .unwrap();
                }
                Event::NotifySubscribe(notify_subscribe) => {
                    println!("GATT server got a notify subscription!");
                    let notifying = Arc::clone(&notifying);
                    notifying.store(true, atomic::Ordering::Relaxed);
                    thread::spawn(move || {
                        let mut count = 0;
                        loop {
                            if !(&notifying).load(atomic::Ordering::Relaxed) {
                                break;
                            };
                            count += 1;
                            println!("GATT server notifying \"hi {}\"!", count);
                            notify_subscribe
                                .clone()
                                .notification
                                .try_send(format!("hi {}", count).into())
                                .unwrap();
                            thread::sleep(Duration::from_secs(2));
                        }
                    });
                }
                Event::NotifyUnsubscribe => {
                    println!("GATT server got a notify unsubscribe!");
                    notifying.store(false, atomic::Ordering::Relaxed);
                }
            };
        }
    };

    let descriptor_handler = async {
        let descriptor_value = Arc::new(Mutex::new(String::from("hi")));
        let mut rx = receiver_descriptor;
        while let Some(event) = rx.next().await {
            match event {
                Event::ReadRequest(read_request) => {
                    println!(
                        "GATT server got a read request with offset {}!",
                        read_request.offset
                    );
                    let value = descriptor_value.lock().unwrap().clone();
                    read_request
                        .response
                        .send(Response::Success(value.clone().into()))
                        .unwrap();
                    println!("GATT server responded with \"{}\"", value);
                }
                Event::WriteRequest(write_request) => {
                    let new_value = String::from_utf8(write_request.data).unwrap();
                    println!(
                        "GATT server got a write request with offset {} and data {}!",
                        write_request.offset, new_value,
                    );
                    *descriptor_value.lock().unwrap() = new_value;
                    write_request
                        .response
                        .send(Response::Success(vec![]))
                        .unwrap();
                }
                _ => panic!("Event not supported for Descriptors!"),
            };
        }
    };

    let (sender_tx, mut receiver_rx) = tokio::sync::mpsc::channel::<PeripheralEvent>(1);

    let mut peripheral = Peripheral::new(sender_tx).await.unwrap();

    tokio::spawn(async move {
        while let Some(event) = receiver_rx.recv().await {
            println!("Peripheral event: {:?}", event);
            handle_updates(event);
        }
    });

    while !peripheral.is_powered().await.unwrap() {}
    println!("Peripheral powered on");

    let service_uuid = Uuid::from_sdp_short_uuid(0x1234_u16);
    peripheral
        .add_service(&Service::new(service_uuid, true, characteristics.clone()))
        .await
        .unwrap();

    peripheral.register_gatt().await.unwrap();

    peripheral
        .start_advertising(ADVERTISING_NAME, &[service_uuid])
        .await
        .unwrap();

    let main_fut = async move {
        println!("Peripheral started advertising");
        let ad_check = async { while !peripheral.is_advertising().await.unwrap() {} };

        let timeout = tokio::time::sleep(ADVERTISING_TIMEOUT);
        futures::join!(ad_check, timeout);
        peripheral.stop_advertising().await.unwrap();
        while peripheral.is_advertising().await.unwrap() {}
        println!("Peripheral stopped advertising");
    };

    futures::join!(characteristic_handler, descriptor_handler, main_fut);
}

pub fn handle_updates(update: PeripheralEvent) {
    match update {
        PeripheralEvent::DidUpdateState { is_powered } => {
            println!("PowerOn: {:?}", is_powered)
        }
        PeripheralEvent::DidStartAdverising { error } => {
            println!("DidStartAdvertising: {:?}", error)
        }
        PeripheralEvent::DidAddService { service, error } => {
            println!("DidAddService: {:?} {:?}", service, error)
        }
        PeripheralEvent::DidSubscribeToCharacteristic {
            client,
            service,
            characteristic,
        } => {
            println!(
                "DidSubscribeToCharacteristic: {:?} {:?} {:?}",
                client, service, characteristic
            )
        }
        PeripheralEvent::DidUnsubscribeFromCharacteristic {
            client,
            service,
            characteristic,
        } => {
            println!(
                "DidUnsubscribeFromCharacteristic: {:?} {:?} {:?}",
                client, service, characteristic
            )
        }
        PeripheralEvent::DidReceiveReadRequest {
            client,
            service,
            characteristic,
        } => {
            println!(
                "DidReceiveReadRequest: {:?} {:?} {:?}",
                client, service, characteristic
            )
        }
        PeripheralEvent::DidReceiveWriteRequest {
            client,
            service,
            characteristic,
            value,
        } => {
            println!(
                "DidReceiveWriteRequest: {:?} {:?} {:?} {:?}",
                client, service, characteristic, value
            )
        }
    }
}
