use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum InternalServiceNotification
{
    FrontendConnected(bool),
    Sleep(bool)
}