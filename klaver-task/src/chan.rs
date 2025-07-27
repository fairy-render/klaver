use crate::async_id::ResourceId;

pub enum Hook {
    Init {
        async_id: ResourceId,
        trigger_source_id: ResourceId,
        ty: String,
    },
    Before(ResourceId),
    After(ResourceId),
    Destroy(ResourceId),
}
