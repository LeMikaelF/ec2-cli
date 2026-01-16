mod local;

pub use local::{
    get_instance, get_linked_instance, list_instances, remove_instance, resolve_instance_name,
    save_instance, InstanceState, State,
};
