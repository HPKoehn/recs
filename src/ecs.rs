extern crate anymap;
use anymap::AnyMap;

use crate::allocation;

type Entity = allocation::GenerationalIndex;
type EntityMap<T> = allocation::GenerationalIndexArray<T>;

struct ECS {
    entitiy_allocator: allocation::GenerationalIndexAllocator,
    entity_components: AnyMap,
}

struct ComponentRegistry {

}