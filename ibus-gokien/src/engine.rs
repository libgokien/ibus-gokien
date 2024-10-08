use core::ptr;
use std::cell::Cell;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::LazyLock;

use gobject_sys::{g_type_class_peek_parent, g_type_is_a, g_type_register_static_simple, GTypeInstance};
use gokien::{GokienEngine, State};
use ribus::c::{self, gboolean, guint, GType, FALSE, TRUE};
use ribus::{g_type_from_class, g_type_from_instance, IBusEngine, IBusEngineClass};
use tracing::{debug, error, instrument};

macro_rules! ibus_engine_class {
    ($class:expr) => {{
        gobject_sys::g_type_check_class_cast($class, ribus::Engine::get_type())
    }};
}

trait IEngine {
    unsafe extern "C" fn process_key_event(
        engine: *mut IBusEngine,
        keyval: guint,
        keycode: guint,
        state: guint,
    ) -> gboolean;
    unsafe extern "C" fn focus_in(engine: *mut IBusEngine);
    unsafe extern "C" fn focus_out(engine: *mut IBusEngine);
    unsafe extern "C" fn reset(engine: *mut IBusEngine);
    unsafe extern "C" fn set_content_type(engine: *mut IBusEngine, purpose: guint, hints: guint);
    unsafe extern "C" fn set_capabilities(engine: *mut IBusEngine, caps: guint);
    unsafe extern "C" fn enable(engine: *mut IBusEngine);
    // unsafe extern "C" fn disable(engine: *mut IBusEngine);

    // Delegate to default IBusEngineClass methods instead
    // unsafe extern "C" fn cursor_up(engine: *mut IBusEngine);
    // unsafe extern "C" fn cursor_down(engine: *mut IBusEngine);
    // unsafe extern "C" fn candidate_clicked(engine: *mut IBusEngine, index: guint, button: guint, state: guint);
    // unsafe extern "C" fn page_up(engine: *mut IBusEngine);
    // unsafe extern "C" fn page_down(engine: *mut IBusEngine);
    // unsafe extern "C" fn property_activate(engine: *mut IBusEngine, prop_name: *const gchar, prop_state: guint);
    #[cfg(feature = "surrounding_text")]
    unsafe extern "C" fn set_surrounding_text(
        engine: *mut IBusEngine,
        text: *mut c::IBusText,
        cursor_index: guint,
        anchor_pos: guint,
    );

    // XXX: maybe ignore below methods with dummy empty function
    // unsafe extern "C" fn set_cursor_location(engine: *mut IBusEngine, x: gint, y: gint, w: gint, h: gint);
    // unsafe extern "C" fn property_show(engine: *mut IBusEngine, prop_name: *const gchar);
    // unsafe extern "C" fn property_hide(engine: *mut IBusEngine, prop_name: *const gchar);

    // extern "C" fn process_hand_writing_event(engine: *mut IBusEngine, coordinates: *const gdouble, coordinates_len: guint);
    // extern "C" fn cancel_hand_writing(engine: *mut IBusEngine, n_strokes: guint);
}

thread_local! {
    pub static PARENT_CLASS: Cell<*mut IBusEngineClass> = const { Cell::new(ptr::null_mut()) };
}

#[repr(C)]
pub struct IBusGokienEngine {
    pub parent: IBusEngine,
    /* members */
    core: GokienEngine,
    disabled: bool,
}

#[repr(C)]
struct IBusGokienEngineClass {
    pub parent: IBusEngineClass,
}

impl IBusGokienEngine {
    #[instrument(level = "trace", skip_all)]
    unsafe extern "C" fn init(this: *mut GTypeInstance, g_class: *mut c_void) {
        debug!(?this, ?g_class);
        Self::is_self(this.cast());
        let this = this.cast::<Self>();
        // SAFETY: this.core should be dangling since zero-initilizing by gobject
        ptr::addr_of_mut!((*this).core).write(GokienEngine::new());
        ptr::addr_of_mut!((*this).disabled).write(false);
        // how to use g_class?
        Self::is_class(g_class.cast());
    }

    fn is_self(this: *mut Self) {
        let ok = unsafe { g_type_is_a(g_type_from_instance!(this), Self::get_type()) != c::FALSE };
        if !ok {
            error!("this is not IBusGokienEngine");
        }
    }

    pub fn is_class(this: *const Self) {
        let ok = unsafe { g_type_is_a(g_type_from_class!(this), Self::get_type()) != c::FALSE };
        if !ok {
            error!("this is not IBusGokienEngineClass");
        }
    }

    pub fn get_type() -> GType {
        static ID: LazyLock<GType> = LazyLock::new(|| unsafe {
            g_type_register_static_simple(
                /* parent_type */ ribus::Engine::get_type(),
                /* type_name */ c"IBusGokienEngine".as_ptr(),
                /* class_size */ size_of::<IBusGokienEngineClass>() as _,
                /* class_init */ Some(IBusGokienEngineClass::init),
                /* instance_size */ size_of::<IBusGokienEngine>() as _,
                /* instance_init */ Some(IBusGokienEngine::init),
                /* flags */ 0,
            )
        });
        *ID
    }

    #[instrument(level = "trace", skip_all)]
    fn update_preedit(&mut self, engine: *mut IBusEngine) {
        let Some(s) = self.core.take_output_as_cstr() else {
            unsafe {
                c::ibus_engine_hide_preedit_text(engine);
            }
            return;
        };
        unsafe {
            let text = c::ibus_text_new_from_static_string(s.as_ptr());
            c::ibus_text_append_attribute(text, c::IBUS_ATTR_TYPE_UNDERLINE, c::IBUS_ATTR_UNDERLINE_SINGLE, 0, -1);
            let len = c::ibus_text_get_length(text);
            c::ibus_engine_update_preedit_text_with_mode(engine, text, len, TRUE, c::IBUS_ENGINE_PREEDIT_COMMIT);
        }
        self.core.replace_output_by_cstr(s);
    }

    #[instrument(level = "trace", skip_all)]
    fn commit_preedit(&mut self, engine: *mut IBusEngine) {
        let Some(s) = self.core.take_output_as_cstr() else {
            return;
        };
        unsafe {
            let text = c::ibus_text_new_from_static_string(s.as_ptr());
            c::ibus_engine_commit_text(engine, text);
            c::ibus_engine_hide_preedit_text(engine);
        }
        self.core.replace_output_by_cstr(s);
        self.core.clear();
    }

    // call more than once in scope is UB.
    unsafe fn assert_is_self<'a>(engine: *mut IBusEngine) -> &'a mut Self {
        let gokien: *mut Self = engine.cast();
        Self::is_self(gokien);
        unsafe { &mut *gokien }
    }
}

impl IBusGokienEngineClass {
    // virtual function overrides go here
    // property and signal definitions go here
    #[instrument(level = "trace", skip_all)]
    unsafe extern "C" fn init(class: *mut c_void, _class_data: *mut c_void) {
        debug!(?class, ?_class_data);

        IBusGokienEngine::is_class(class.cast());
        let class = class.cast::<Self>();

        let parent: *mut IBusEngineClass = g_type_class_peek_parent(class.cast()).cast();
        ribus::Engine::is_class(parent.cast());
        PARENT_CLASS.set(parent);

        // virtual function overrides go here

        // NOTE: `parent` should be let untouched to get default impl
        let engine_class: *mut IBusEngineClass = ibus_engine_class!(class.cast()).cast();
        let parent = &mut *engine_class;
        parent.process_key_event.replace(IBusGokienEngine::process_key_event);
        parent.focus_in.replace(IBusGokienEngine::focus_in);
        parent.focus_out.replace(IBusGokienEngine::focus_out);
        parent.reset.replace(IBusGokienEngine::reset);
        parent.set_content_type.replace(IBusGokienEngine::set_content_type);
        parent.enable.replace(IBusGokienEngine::enable);
        parent.set_capabilities.replace(IBusGokienEngine::set_capabilities);
        #[cfg(feature = "surrounding_text")]
        parent
            .set_surrounding_text
            .replace(IBusGokienEngine::set_surrounding_text);

        // let g_class: *mut GObjectClass = class.cast();
        // HACK: constructor nonsense: <https://docs.gtk.org/gobject/concepts.html#object-instantiation>
        // (*g_class).constructor = Some(IBusGokienEngine::constructor);
        // If you need to perform object initialization steps after
        // all construct properties have been set.
        // (*g_class).constructed = Some(IBusGokienEngine::constructed);
        // (*g_class).finalize = Some(IBusGokienEngine::finalize);
        // let io_class: *mut ribus::ObjectClass = class.cast();
        // (*io_class).destroy = Some(42);
    }
}

impl IEngine for IBusGokienEngine {
    #[instrument(level = "debug", skip(engine, _kcode))]
    unsafe extern "C" fn process_key_event(
        engine: *mut IBusEngine,
        ksym: guint,
        _kcode: guint,
        state: guint,
    ) -> gboolean {
        let gokien = Self::assert_is_self(engine);

        if gokien.disabled {
            return FALSE;
        }

        let processed = gokien.core.process_key(ksym, state);

        match gokien.core.state {
            State::Typing => {
                gokien.update_preedit(engine);
            }
            State::PreeditCommitting => {
                debug!(output = %gokien.core.get_output());
                gokien.commit_preedit(engine);
                gokien.core.state = State::Typing;
            }
            State::Interrupting => {
                gokien.commit_preedit(engine);
                // FIXME: maybe we want to disable TELEX here
            }
            State::Backspacing => {
                gokien.core.state = State::Typing;
                let ok = gokien.core.handle_backspace();
                gokien.update_preedit(engine);
                return if ok { TRUE } else { FALSE };
            }
        }

        if processed {
            TRUE
        } else {
            FALSE
        }
    }

    #[instrument(level = "trace", skip_all)]
    unsafe extern "C" fn focus_in(engine: *mut IBusEngine) {
        let gokien = Self::assert_is_self(engine);

        let disabled = ribus::Engine::should_be_disable(engine);
        debug!(?disabled);
        gokien.disabled = disabled;

        if cfg!(feature = "surrounding_text") {
            let mut cursor_index = 0;
            let mut anchor_pos = 0;
            let mut text: *mut c::IBusText = ptr::null_mut();
            c::ibus_engine_get_surrounding_text(engine, &mut text, &mut cursor_index, &mut anchor_pos);
            let text = c::ibus_text_get_text(text);
            let text = std::ffi::CStr::from_ptr(text);
            debug!(?text, cursor_index, anchor_pos);
        }

        (*PARENT_CLASS.get()).focus_in.map(|f| f(engine));
    }

    #[instrument(level = "trace", skip_all)]
    unsafe extern "C" fn focus_out(engine: *mut IBusEngine) {
        let gokien = Self::assert_is_self(engine);
        gokien.commit_preedit(engine);
        gokien.core.reset();
        c::ibus_engine_hide_preedit_text(engine);
        (*PARENT_CLASS.get()).focus_out.map(|f| f(engine));
    }

    #[instrument(level = "trace", skip_all)]
    unsafe extern "C" fn reset(engine: *mut IBusEngine) {
        let gokien = Self::assert_is_self(engine);
        gokien.core.reset();
        (*PARENT_CLASS.get()).reset.map(|f| f(engine));
    }

    #[instrument(level = "trace", skip_all)]
    unsafe extern "C" fn set_content_type(engine: *mut IBusEngine, purpose: guint, _hints: guint) {
        let gokien = Self::assert_is_self(engine);
        gokien.disabled = ribus::Engine::invalid_input_context(purpose);
    }

    #[instrument(level = "trace", skip_all)]
    unsafe extern "C" fn set_capabilities(_engine: *mut IBusEngine, caps: guint) {
        // some terminal emulators don't have preedit.
        let _has_preedit_text = caps & c::IBUS_CAP_PREEDIT_TEXT != 0;
        // almost all clients shall be able to be focused.
        let has_focus = caps & c::IBUS_CAP_FOCUS != 0;
        // many clients support surrounding text feature.
        let has_surrounding_text = caps & c::IBUS_CAP_SURROUNDING_TEXT != 0;

        if !has_focus {
            error!("client is not able to get focus");
        }

        if !has_surrounding_text {
            error!("client doesn't support surrounding text");
        }
    }

    #[instrument(level = "trace", skip_all)]
    unsafe extern "C" fn enable(engine: *mut IBusEngine) {
        // > It is also used to tell the input-context that the engine will utilize surrounding-text.
        // > In that case, it must be called in "enable" handler, with both text and cursor set to NULL.
        c::ibus_engine_get_surrounding_text(engine, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
    }

    // FIXME: this function cannot receive anything from clients.
    //        At least tested in firefox and sublimetext on pop-os.
    #[instrument(level = "trace", skip_all)]
    #[cfg(feature = "surrounding_text")]
    unsafe extern "C" fn set_surrounding_text(
        _engine: *mut IBusEngine,
        text: *mut c::IBusText,
        cursor_index: guint,
        anchor_pos: guint,
    ) {
        let text = c::ibus_text_get_text(text);
        let text = core::ffi::CStr::from_ptr(text);
        debug!(?text, cursor_index, anchor_pos);
    }
}
