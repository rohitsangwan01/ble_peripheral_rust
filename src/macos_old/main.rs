mod ffi;
mod peripheral_delegate;
mod peripheral_manager;

use std::{ffi::CString, thread};

use objc2::{msg_send_id, rc::Retained, runtime::AnyObject, ClassType};
use objc2_core_bluetooth::{CBManagerState, CBPeripheralManager};
use objc2_foundation::{NSDictionary, NSString};
use peripheral_delegate::{PeripheralDelegate, PeripheralDelegateEvent};
use peripheral_manager::{check_permission, PeripheralManager};
use tokio::{runtime, sync::mpsc::channel};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    if !check_permission() {
        println!("Bluetooth Permission not granted")
    }

    let (sender, mut receiver) = channel::<PeripheralDelegateEvent>(256);

    let peripheral_delegate = PeripheralDelegate::new(sender);
    let label = CString::new("CBqueue").unwrap();
    let queue = unsafe { ffi::dispatch_queue_create(label.as_ptr(), ffi::DISPATCH_QUEUE_SERIAL) };
    let queue: *mut AnyObject = queue.cast();
    let peripheral_manager: Retained<CBPeripheralManager> = unsafe {
        msg_send_id![CBPeripheralManager::alloc(), initWithDelegate: &*peripheral_delegate, queue: queue]
    };

    let is_advertising = unsafe { peripheral_manager.isAdvertising() };

    println!("Isadvertising: {}", is_advertising);

    let advertisement_data: Option<&NSDictionary<NSString, AnyObject>> = None;

    unsafe {
        peripheral_manager.startAdvertising(advertisement_data);
    }

    while let Some(update) = receiver.recv().await {
        handle_update(update);
    }
}

fn handle_update(update: PeripheralDelegateEvent) {
    match update {
        PeripheralDelegateEvent::DidUpdateState { state } => {
            println!("BleOn: {}", state == CBManagerState::PoweredOn)
        }
    }
}
