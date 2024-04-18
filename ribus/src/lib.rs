mod ffi;
mod iter;

use std::ffi::CStr;
use std::mem::transmute;
use std::ptr;

pub use c::{
    IBusBusNameFlag as NameFlag, IBusEngine, IBusEngineClass, IBusObject as Object, IBusObjectClass as ObjectClass,
};
pub use ffi::ibus as c;
use glib_sys::{g_string_free, g_string_sized_new};
use gobject_sys::{g_object_unref, g_signal_connect_object, g_type_is_a};
use iter::EngineIter;
use libdbus_sys::DBusRequestNameReply;
use tracing::info;

#[macro_export]
macro_rules! g_type_from_class {
    ($class:expr) => {
        (*($class as *const gobject_sys::GTypeClass)).g_type
    };
}

#[macro_export]
macro_rules! g_type_from_instance {
    ($obj:ident) => {
        $crate::g_type_from_class!((*($obj as *const gobject_sys::GTypeInstance)).g_class)
    };
}

macro_rules! drop_gobject {
    ($Ty:ty) => {
        impl Drop for $Ty {
            fn drop(&mut self) {
                unsafe { g_object_unref(self.0.cast()) }
            }
        }
    };
}

pub fn main() {
    unsafe {
        c::ibus_main();
    }
}

// todo implement Drop with g_object_unref
// https://gtk-rs.org/gtk4-rs/stable/latest/book/g_object_concepts.html
#[repr(transparent)]
pub struct Bus(*mut c::IBusBus);

drop_gobject!(Bus);
drop_gobject!(Factory);
drop_gobject!(Component);

impl Bus {
    pub fn new() -> Option<Self> {
        unsafe {
            c::ibus_init();
            let bus = c::ibus_bus_new();
            match c::ibus_bus_is_connected(bus) {
                c::FALSE => None,
                _ => Some(Self(bus)),
            }
        }
    }

    pub fn list_engines(&self) -> EngineIter {
        unsafe {
            let list = c::ibus_bus_list_engines(self.0);
            EngineIter::new(list)
        }
    }

    pub fn hello(&self) -> &CStr {
        unsafe { CStr::from_ptr(c::ibus_bus_hello(self.0)) }
    }

    pub fn get_global_engine(&self) -> Option<EngineDesc> {
        unsafe {
            let desc = c::ibus_bus_get_global_engine(self.0);
            if desc.is_null() {
                return None;
            }
            Some(EngineDesc(desc))
        }
    }

    pub fn request_name(&self, name: &CStr, name_flag: NameFlag) -> Option<DBusRequestNameReply> {
        unsafe {
            match c::ibus_bus_request_name(self.0, name.as_ptr(), name_flag as _) {
                0 => None,
                flag => Some(transmute(flag)),
            }
        }
    }

    #[must_use]
    pub fn register_component(&self, component: &Component) -> bool {
        unsafe { c::FALSE != c::ibus_bus_register_component(self.0, component.0) }
    }

    pub fn get_config(&self) {
        unimplemented!()
    }

    extern "C" fn quit() {
        info!("bus quit");
        unsafe {
            c::ibus_quit();
        }
    }

    pub fn register_disconnected_signal(&self) {
        unsafe {
            g_signal_connect_object(
                /* instance */ self.0.cast(),
                /* detailed_signal */ c"disconnected".as_ptr(),
                /* c_handler */ Some(Self::quit),
                /* gobject */ ptr::null_mut(),
                /* connect_flag */ gobject_sys::G_CONNECT_DEFAULT,
            );
        }
    }
}

#[repr(transparent)]
pub struct Factory(*mut c::IBusFactory);

// todo implement Drop with g_object_unref
// https://gtk-rs.org/gtk4-rs/stable/latest/book/g_object_concepts.html
impl Factory {
    pub fn new(bus: &Bus) -> Self {
        unsafe {
            let dbus = c::ibus_bus_get_connection(bus.0);
            let fa = c::ibus_factory_new(dbus);
            Self(fa)
        }
    }

    pub fn add_engine(&mut self, engine_name: &CStr, engine_type: c::GType) {
        unsafe {
            c::ibus_factory_add_engine(self.0, engine_name.as_ptr(), engine_type);
        }
    }

    // todo this maybe moved ownership to caller
    pub fn create_engine(&mut self, engine: &CStr) -> Option<&mut Engine> {
        unsafe {
            let e = c::ibus_factory_create_engine(self.0, engine.as_ptr());
            if e.is_null() {
                return None;
            }
            Some(&mut *(e as *mut Engine))
        }
    }
}

// TODO: add Drop for this
#[repr(transparent)]
pub struct Component(*mut c::IBusComponent);

impl Component {
    pub fn new_from_file(path: &CStr) -> Self {
        unsafe {
            let comp = c::ibus_component_new_from_file(path.as_ptr());
            Self(comp)
        }
    }

    pub fn get_name(&self) -> &CStr {
        unsafe {
            let name = c::ibus_component_get_name(self.0);
            CStr::from_ptr(name)
        }
    }

    pub fn add_engine(&mut self, engine: EngineDesc) {
        unsafe {
            c::ibus_component_add_engine(self.0, engine.0);
        }
    }

    pub fn get_engines(&self) -> iter::EngineIter {
        unsafe {
            let list = c::ibus_component_get_engines(self.0);
            iter::EngineIter::new(list)
        }
    }

    pub fn output(&self) {
        unsafe {
            let gs = g_string_sized_new(1024);
            if gs.is_null() {
                return;
            }
            c::ibus_component_output(self.0, gs.cast(), 0);
            let s = std::slice::from_raw_parts((*gs).str.cast::<u8>(), (*gs).len);
            println!("{}", std::str::from_utf8(s).unwrap());
            g_string_free(gs, !c::FALSE);
        }
    }
}

// The recommended way to load engine description data is using
// `Component::new_from_file` to load a component file, which also includes
// engine description data.
#[repr(transparent)]
pub struct EngineDesc(*mut c::IBusEngineDesc);

impl EngineDesc {
    pub fn get_name(&self) -> &CStr {
        unsafe {
            let name = c::ibus_engine_desc_get_name(self.0);
            CStr::from_ptr(name)
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Engine(IBusEngine);

impl Engine {
    pub fn new() -> Self {
        panic!("use Factory::create_engine instead");
    }

    pub fn get_name(&self) -> &CStr {
        todo!()
    }

    pub fn is_self(this: *const Self) -> bool {
        unsafe { g_type_is_a(g_type_from_instance!(this), Self::get_type()) != c::FALSE }
    }

    pub fn is_class(this: *const Self) -> bool {
        unsafe { g_type_is_a(g_type_from_class!(this), Self::get_type()) != c::FALSE }
    }

    pub fn get_type() -> c::GType {
        unsafe { c::ibus_engine_get_type() }
    }

    pub fn as_ptr(&mut self) -> *mut IBusEngine {
        &mut self.0 as _
    }

    #[inline]
    pub fn invalid_input_context(purpose: c::guint) -> bool {
        match purpose {
            c::IBUS_INPUT_PURPOSE_FREE_FORM | c::IBUS_INPUT_PURPOSE_ALPHA | c::IBUS_INPUT_PURPOSE_NAME => false,
            _ => true,
        }
    }

    #[must_use]
    pub fn should_be_disable(engine: *mut IBusEngine) -> bool {
        let mut purpose = c::IBUS_INPUT_PURPOSE_FREE_FORM;
        let mut hints = c::IBUS_INPUT_HINT_NONE;
        unsafe {
            c::ibus_engine_get_content_type(engine, &mut purpose, &mut hints);
        }
        Self::invalid_input_context(purpose)
    }
}
