use crate::gatt::peripheral_event::PeripheralEvent;

use super::{mac_extensions::UuidHelper, mac_utils};
use objc2::{
    declare_class, msg_send_id, mutability, rc::Retained, runtime::AnyObject, ClassType,
    DeclaredClass,
};
use objc2_core_bluetooth::{
    CBATTError, CBATTRequest, CBCentral, CBCharacteristic, CBManagerState, CBPeripheralManager,
    CBPeripheralManagerDelegate, CBService,
};
use objc2_foundation::{NSArray, NSData, NSError, NSObject, NSObjectProtocol};
use std::{cell::RefCell, ffi::CString, fmt::Debug, sync::Arc};
use tokio::sync::{mpsc::Sender, oneshot};

declare_class!(
    #[derive(Debug)]
    pub struct PeripheralDelegate;

    unsafe impl ClassType for PeripheralDelegate {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "PeripheralManagerDelegate";
    }

    impl DeclaredClass for PeripheralDelegate {
        type Ivars = (Sender<PeripheralEvent>, RefCell<Option<Retained<CBPeripheralManager>>>);
    }

    unsafe impl NSObjectProtocol for PeripheralDelegate {}

    unsafe impl CBPeripheralManagerDelegate for PeripheralDelegate {
        #[method(peripheralManagerDidUpdateState:)]
         fn delegate_peripheralmanagerdidupdatestate(&self, peripheral: &CBPeripheralManager){
                let state = unsafe { peripheral.state() };
                self.send_event(PeripheralEvent::DidUpdateState { is_powered : state == CBManagerState::PoweredOn });
         }

        #[method(peripheralManagerDidStartAdvertising:error:)]
        fn delegate_peripheralmanagerdidstartadvertising_error(&self, _: &CBPeripheralManager,error: Option<&NSError>){
            let mut error_desc: Option<String> = None;
            if let Some(error) = error {
                error_desc = Some(error.localizedDescription().to_string());
            }
            self.send_event(PeripheralEvent::DidStartAdverising { error: error_desc });
        }

        #[method(peripheralManager:didAddService:error:)]
         fn delegate_peripheralmanager_didaddservice_error(&self, _: &CBPeripheralManager,service: &CBService, error: Option<&NSError>){
            let mut error_desc: Option<String> = None;
            if let Some(error) = error {
                error_desc = Some(error.localizedDescription().to_string());
            }
            self.send_event(PeripheralEvent::DidAddService {
                service: service.get_uuid(),
                error: error_desc
            });
        }

        #[method(peripheralManager:central:didSubscribeToCharacteristic:)]
         fn delegate_peripheralmanager_central_didsubscribetocharacteristic(
            &self,
            _: &CBPeripheralManager,
            central: &CBCentral,
            characteristic: &CBCharacteristic,
        ){
            unsafe{
                let service: Option<Retained<CBService>> = characteristic.service();
                if service.is_none() {
                    return;
                }
                self.send_event(PeripheralEvent::DidSubscribeToCharacteristic {
                    client: central.identifier().to_string(),
                    service: characteristic.service().unwrap().get_uuid(),
                    characteristic: characteristic.get_uuid(),
                });
            }
        }

        #[method(peripheralManager:central:didUnsubscribeFromCharacteristic:)]
         fn delegate_peripheralmanager_central_didunsubscribefromcharacteristic(
            &self,
            _: &CBPeripheralManager,
            central: &CBCentral,
            characteristic: &CBCharacteristic,
        ){  unsafe{
            let service: Option<Retained<CBService>> = characteristic.service();
            if service.is_none() {
                return;
            }
            self.send_event(PeripheralEvent::DidUnsubscribeFromCharacteristic {
                client: central.identifier().to_string(),
                service: characteristic.service().unwrap().get_uuid(),
                characteristic: characteristic.get_uuid(),
            });
        }}

        #[method(peripheralManager:didReceiveReadRequest:)]
         fn delegate_peripheralmanager_didreceivereadrequest(
            &self,
            _: &CBPeripheralManager,
            request: &CBATTRequest,
        ){
            unsafe{
                let service = request.characteristic().service();
                if service.is_none() {
                    return;
                }
                let central = request.central();
                let characteristic = request.characteristic();

                let (resp_tx, resp_rx) = oneshot::channel::<Vec<u8>>();
                self.send_and_respond(
                    PeripheralEvent::DidReceiveReadRequest{
                        client: central.identifier().to_string(),
                        service: characteristic.service().unwrap().get_uuid(),
                        characteristic: characteristic.get_uuid(),
                        responder: resp_tx,
                    },
                    request,
                    resp_rx,
                );
            }
        }

        #[method(peripheralManager:didReceiveWriteRequests:)]
         fn delegate_peripheralmanager_didreceivewriterequests(
            &self,
            _: &CBPeripheralManager,
            requests: &NSArray<CBATTRequest>,
        ){
            for request in requests {
                unsafe{
                    let service = request.characteristic().service();
                    if service.is_none() {
                        return;
                    }
                    let mut value: Vec<u8> = Vec::new();

                    if let Some(ns_data) = request.value() {
                       value = ns_data.bytes().to_vec();
                    }

                    let central = request.central();
                    let characteristic = request.characteristic();
                    self.send_event(PeripheralEvent::DidReceiveWriteRequest{
                        client: central.identifier().to_string(),
                        service: characteristic.service().unwrap().get_uuid(),
                        characteristic: characteristic.get_uuid(),
                        value: value,
                    });
                }
            }
        }
    }
);

impl PeripheralDelegate {
    pub fn new(
        sender: Sender<PeripheralEvent>,
    ) -> (
        Retained<CBPeripheralManager>,
        Arc<Retained<PeripheralDelegate>>,
    ) {
        let this = PeripheralDelegate::alloc().set_ivars((sender, RefCell::new(None)));
        let delegate: Arc<Retained<PeripheralDelegate>> =
            Arc::new(unsafe { msg_send_id![super(this), init] });
        let label: CString = CString::new("CBqueue").unwrap();
        let queue: *mut std::ffi::c_void = unsafe {
            mac_utils::dispatch_queue_create(label.as_ptr(), mac_utils::DISPATCH_QUEUE_SERIAL)
        };
        let queue: *mut AnyObject = queue.cast();
        let peripheral_manager_delegate: Retained<CBPeripheralManager> = unsafe {
            msg_send_id![CBPeripheralManager::alloc(), initWithDelegate: &**delegate, queue: queue]
        };

        // Store CBPeripheralManager delegate within PeripheralDelegate to respond on requests
        // However, it creates a circular reference, which could potentially lead to memory leaks if not managed carefully
        delegate
            .ivars()
            .1
            .borrow_mut()
            .replace(peripheral_manager_delegate.clone());

        return (peripheral_manager_delegate, delegate);
    }

    pub fn get_peripheral_manager(&self) -> Retained<CBPeripheralManager> {
        return self.ivars().1.borrow().clone().unwrap();
    }

    fn send_event(&self, event: PeripheralEvent) {
        let sender = self.ivars().0.clone();
        futures::executor::block_on(async {
            if let Err(e) = sender.send(event).await {
                log::error!("Error sending delegate event: {}", e);
            }
        });
    }

    fn send_and_respond(
        &self,
        event: PeripheralEvent,
        request: &CBATTRequest,
        resp_rx: oneshot::Receiver<Vec<u8>>,
    ) {
        let sender = self.ivars().0.clone();

        futures::executor::block_on(async {
            // Send to Listnere
            if let Err(e) = sender.send(event).await {
                log::error!("Error sending delegate event: {}", e);
                return;
            }

            // Wait for response
            let result = resp_rx.await;
            unsafe {
                if result.is_ok() {
                    request.setValue(Some(&NSData::from_vec(result.unwrap())));
                } else {
                    request.setValue(None);
                }

                // Update Manager
                self.get_peripheral_manager()
                    .respondToRequest_withResult(request, CBATTError::Success);
            }
        });
    }
}

impl Drop for PeripheralDelegate {
    fn drop(&mut self) {
        // Clear the reference to CBPeripheralManager, and Remove delegate
        if let Some(manager) = self.ivars().1.borrow_mut().take() {
            unsafe {
                manager.setDelegate(None);
                log::debug!("Delegate removed")
            }
        }
        log::debug!("PeripheralDelegate dropped");
    }
}
