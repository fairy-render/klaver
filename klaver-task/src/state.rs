use crate::async_id::ResourceId;

pub struct AsyncState {
    current_async_id: ResourceId,
    execution_id: ResourceId,
}
