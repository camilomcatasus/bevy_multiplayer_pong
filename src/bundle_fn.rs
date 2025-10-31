use bevy::ecs::bundle::DynamicBundle;
use bevy::ecs::component::*;
use bevy::prelude::*;
use bevy::ptr::MovingPtr;

pub trait Thunk: FnOnce(&mut EntityWorldMut) + Send + Sync + 'static {}
impl<F: FnOnce(&mut EntityWorldMut) + Send + Sync + 'static> Thunk for F {}

pub struct BundleFn<F: Thunk>(pub F);

unsafe impl<F: Thunk> Bundle for BundleFn<F> {
    fn component_ids(_: &mut ComponentsRegistrator, _: &mut impl FnMut(ComponentId)) {}

    fn get_component_ids(_: &Components, _: &mut impl FnMut(Option<ComponentId>)) {}
}

impl<F: Thunk> DynamicBundle for BundleFn<F> {
    type Effect = BundleFn<F>;
    unsafe fn get_components(
        _ptr: MovingPtr<'_, Self>,
        _func: &mut impl FnMut(StorageType, bevy::ptr::OwningPtr<'_>),
    ) {}

    unsafe fn apply_effect(ptr: bevy::ptr::MovingPtr<'_, std::mem::MaybeUninit<Self>>, entity: &mut EntityWorldMut) {
        unsafe {
            (ptr.read().assume_init().0)(entity);
        }
    }
}
