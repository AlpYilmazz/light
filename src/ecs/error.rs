
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StorageFault {
    EntityNotCreated,
    ComponentNotRegistered,
    NoComponentOnEntity,
    ComponentUninitOnEntity,
}