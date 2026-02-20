#[derive(Debug)]
pub enum SimulationError {
    BadTimeStep,
    BadPrintIndex,
    ThreadConnectionError,
    ReceiverClosedError,
    InvalidChannelCommunication,
}

impl From<crate::FloatConversionError> for SimulationError {
    fn from(_value: crate::FloatConversionError) -> Self {
        Self::BadTimeStep
    }
}

pub enum BroadcastError {
    UnlinkedChannel,    // fails if sys_channel isnt linked.
    DeallocatedChannel, // fails if a message is sent to a channel that's been deallocated.
    WrongInit,          // fails if the first message to a new thread isnt GoSynced
    BadRuntimeSignal,   // fails if a signal that doesnt make sense (eg Go) is received
}

impl From<BroadcastError> for SimulationError {
    fn from(value: BroadcastError) -> Self {
        match value {
            BroadcastError::UnlinkedChannel => Self::ThreadConnectionError,
            BroadcastError::DeallocatedChannel => Self::ReceiverClosedError,
            BroadcastError::WrongInit => Self::InvalidChannelCommunication,
            BroadcastError::BadRuntimeSignal => Self::InvalidChannelCommunication,
        }
    }
}

impl<_T> From<std::sync::mpsc::SendError<_T>> for BroadcastError {
    fn from(_value: std::sync::mpsc::SendError<_T>) -> Self {
        Self::DeallocatedChannel
    }
}
