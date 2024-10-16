use crate::Error;

use super::{ffi, peripheral_delegate::PeripheralDelegate};
use log::{trace, warn};
use objc2::{msg_send_id, rc::Retained, runtime::AnyObject, ClassType};
use objc2_core_bluetooth::{CBManager, CBManagerAuthorization, CBPeripheralManager};
use std::{ffi::CString, sync::Arc};

pub struct PeripheralManager {
    peripheral_manager: Retained<CBPeripheralManager>,
}

impl PeripheralManager {
    pub fn new(peripheral_delegate: Arc<Retained<PeripheralDelegate>>) -> Result<Self, Error> {
        let authorization = unsafe { CBManager::authorization_class() };
        if authorization != CBManagerAuthorization::AllowedAlways
            && authorization != CBManagerAuthorization::NotDetermined
        {
            warn!("Authorization status {:?}", authorization);
            return Err(Error::from_type(crate::ErrorType::PermissionDenied));
        } else {
            trace!("Authorization status {:?}", authorization);
        }

        let label: CString = CString::new("CBqueue").unwrap();
        let queue: *mut std::ffi::c_void =
            unsafe { ffi::dispatch_queue_create(label.as_ptr(), ffi::DISPATCH_QUEUE_SERIAL) };
        let queue: *mut AnyObject = queue.cast();
        let peripheral_manager: Retained<CBPeripheralManager> = unsafe {
            msg_send_id![CBPeripheralManager::alloc(), initWithDelegate: &**peripheral_delegate, queue: queue]
        };
        Ok(Self { peripheral_manager })
    }

    // async fn wait_for_message(&mut self) {
    //     select! {
    //         delegate_msg = self.delegate_receiver.next() => {}
    //     }
    // }

    pub fn is_advertising(&self) -> bool {
        unsafe { self.peripheral_manager.isAdvertising() }
    }
}

// pub fn run_corebluetooth_thread(
//     event_sender: Sender<PeripheralDelegateEvent>,
// ) -> Sender<PeripheralDelegateEvent> {
//     let (sender, receiver) = channel::<PeripheralDelegateEvent>(256);
//     // CoreBluetoothInternal is !Send, so we need to keep it on a single thread.
//     thread::spawn(move || {
//         let runtime = runtime::Builder::new_current_thread().build().unwrap();
//         runtime.block_on(async move {
//             let mut cbi = PeripheralManager::new(event_sender);
//             loop {
//                 cbi.wait_for_message().await;
//             }
//         })
//     });
//     sender
// }

pub fn check_permission() -> bool {
    let authorization = unsafe { CBManager::authorization_class() };
    return authorization == CBManagerAuthorization::AllowedAlways
        || authorization == CBManagerAuthorization::NotDetermined;
}
