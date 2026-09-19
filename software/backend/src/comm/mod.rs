use tokio::sync::{mpsc::{self, UnboundedReceiver}, watch};

use crate::comm::{external_commands::Command, internal_notifications::InternalServiceNotification};

pub(crate) mod external_commands;
pub(crate) mod internal_notifications;

#[derive(Clone)]
pub struct InternalState
{
    pub front_end_connected:bool,
    pub sleeping:bool
}

pub struct CommunicationHub
{
    state:InternalState,
    internal_service_notification_receiver:mpsc::UnboundedReceiver<InternalServiceNotification>, //Cannot clone, instead, make a new receiver by subscribing to the sender
    internal_service_state_receiver:watch::Receiver<InternalState>, //Cannot clone, instead, make a new receiver by subscribing to the sender
    spoke:CommunicationSpoke
}

#[derive(Clone)]
pub struct CommunicationSpoke
{
    internal_service_notification_sender:mpsc::UnboundedSender<InternalServiceNotification>,
    internal_service_state_sender:watch::Sender<InternalState>,
    external_command_sender:mpsc::UnboundedSender<Command>
}

impl CommunicationHub
{
    pub fn new()->(CommunicationHub,mpsc::UnboundedReceiver<Command>)
    {
        let state=InternalState{
            front_end_connected:false,
            sleeping:false
        };

        let (
            internal_service_notification_sender,
            internal_service_notification_receiver
        ) = mpsc::unbounded_channel::<InternalServiceNotification>();
        
        let (
            internal_service_state_sender,
            internal_service_state_receiver
        ) = watch::channel::<InternalState>(state.clone());

        //receiver doesn't implement clone so can't pass it in to a service.
        let (
            external_command_sender,
            external_command_receiver
        ) = tokio::sync::mpsc::unbounded_channel::<Command>();


        let spoke = CommunicationSpoke {
            internal_service_notification_sender,
            internal_service_state_sender,
            external_command_sender
        };

        (
            CommunicationHub{
                state,
                internal_service_notification_receiver,
                internal_service_state_receiver,
                spoke
            },
            external_command_receiver
        )
    }   

    async fn internal_service_notification_processing_loop(&mut self)
    {
        loop{
            match self.internal_service_notification_receiver.recv().await
            {
                Some(notification) => {
                    match notification
                    {
                        InternalServiceNotification::FrontendConnected(front_end_connected) => {
                            self.state.front_end_connected=front_end_connected;
                        },
                        InternalServiceNotification::Sleep(sleeping) => {
                            self.state.sleeping=sleeping;
                        },
                    }
                },
                None => (),
            };

            match self.spoke.internal_service_state_sender.send(self.state.clone())
            {
                Ok(_) => (),
                Err(e) => {eprintln!("{:?}",e)},
            }
        }
    }

    pub async fn start(&mut self)->Result<(),Box<dyn std::error::Error + Send + Sync>>
    {
        self.internal_service_notification_processing_loop().await;

        Ok(())
    }

    pub fn spoke(&self)->&CommunicationSpoke
    {
        &self.spoke
    }
}

impl CommunicationSpoke
{
    pub fn notify_of_state_change(&self, state_change:InternalServiceNotification)->bool
    {
        match self.internal_service_notification_sender.send(state_change)
        {
            Ok(_) => true,
            Err(e) => {
                eprintln!("{:?}",e);
                false
            },
        }
    }

    pub fn send_external_command(&self, command:Command)->bool
    {
        match self.external_command_sender.send(command)
        {
            Ok(_) => true,
            Err(e) => {
                eprintln!("{:?}",e);
                false
            },
        }
    }

    pub fn get_internal_state_receiver(&self)->watch::Receiver<InternalState>
    {
        self.internal_service_state_sender.subscribe()
    }
}