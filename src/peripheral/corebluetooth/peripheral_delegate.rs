use super::mac_extensions::UuidHelper;
use objc2::{declare_class, msg_send_id, mutability, rc::Retained, ClassType, DeclaredClass};
use objc2_core_bluetooth::{
    CBATTRequest, CBCentral, CBCharacteristic, CBManagerState, CBPeripheralManager,
    CBPeripheralManagerDelegate, CBService,
};
use objc2_foundation::{NSArray, NSError, NSObject, NSObjectProtocol};
use std::fmt::Debug;
use tokio::sync::mpsc::{self, Sender};
use uuid::Uuid;

pub enum PeripheralDelegateEvent {
    DidUpdateState {
        state: CBManagerState,
    },
    DidStartAdverising {
        error: Option<String>,
    },
    DidAddService {
        service: Uuid,
        error: Option<String>,
    },
    DidSubscribeToCharacteristic {
        client: String,
        service: Uuid,
        characteristic: Uuid,
    },
    DidUnsubscribeFromCharacteristic {
        client: String,
        service: Uuid,
        characteristic: Uuid,
    },
    DidReceiveReadRequest {
        client: String,
        service: Uuid,
        characteristic: Uuid,
    },
    DidReceiveWriteRequest {
        client: String,
        service: Uuid,
        characteristic: Uuid,
    },
}

declare_class!(
    #[derive(Debug)]
    pub struct PeripheralDelegate;

    unsafe impl ClassType for PeripheralDelegate {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "PeripheralManagerDelegate";
    }

    impl DeclaredClass for PeripheralDelegate {
        type Ivars = Sender<PeripheralDelegateEvent>;
    }

    unsafe impl NSObjectProtocol for PeripheralDelegate {}

    unsafe impl CBPeripheralManagerDelegate for PeripheralDelegate {
        #[method(peripheralManagerDidUpdateState:)]
         fn delegate_peripheralmanagerdidupdatestate(&self, peripheral: &CBPeripheralManager){
                let state = unsafe { peripheral.state() };
                self.send_event(PeripheralDelegateEvent::DidUpdateState { state });
         }

        #[method(peripheralManagerDidStartAdvertising:error:)]
        fn delegate_peripheralmanagerdidstartadvertising_error(&self, _: &CBPeripheralManager,error: Option<&NSError>){
            let mut error_desc: Option<String> = None;
            if let Some(error) = error {
                error_desc = Some(error.localizedDescription().to_string());
            }
            self.send_event(PeripheralDelegateEvent::DidStartAdverising { error: error_desc });
        }

        #[method(peripheralManager:didAddService:error:)]
         fn delegate_peripheralmanager_didaddservice_error(&self, _: &CBPeripheralManager,service: &CBService, error: Option<&NSError>){
            let mut error_desc: Option<String> = None;
            if let Some(error) = error {
                error_desc = Some(error.localizedDescription().to_string());
            }
            self.send_event(PeripheralDelegateEvent::DidAddService {
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
                self.send_event(PeripheralDelegateEvent::DidSubscribeToCharacteristic {
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
            self.send_event(PeripheralDelegateEvent::DidUnsubscribeFromCharacteristic {
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
                self.send_event(PeripheralDelegateEvent::DidReceiveReadRequest{
                    client: central.identifier().to_string(),
                    service: characteristic.service().unwrap().get_uuid(),
                    characteristic: characteristic.get_uuid(),
                });
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
                    let central = request.central();
                    let characteristic = request.characteristic();
                    self.send_event(PeripheralDelegateEvent::DidReceiveWriteRequest{
                        client: central.identifier().to_string(),
                        service: characteristic.service().unwrap().get_uuid(),
                        characteristic: characteristic.get_uuid(),
                    });
                }
            }
        }
    }
);

impl PeripheralDelegate {
    pub fn new(sender: mpsc::Sender<PeripheralDelegateEvent>) -> Retained<Self> {
        let this = PeripheralDelegate::alloc().set_ivars(sender);
        unsafe { msg_send_id![super(this), init] }
    }

    fn send_event(&self, event: PeripheralDelegateEvent) {
        let sender = self.ivars().clone();
        futures::executor::block_on(async {
            if let Err(e) = sender.send(event).await {
                println!("Error sending delegate event: {}", e);
            }
        });
    }
}
