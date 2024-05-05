#[cfg(windows)]
compile_eror!("ibus is not supported on Windows");

mod engine;

use std::borrow::Cow;
use std::env;
use std::ffi::{CStr, CString, OsStr};
use std::path::{Path, PathBuf};

use argh::FromArgs;
use engine::IBusGokienEngine;
use ribus::{Bus, Component, Factory, NameFlag};
use tracing::{error, info};

#[derive(FromArgs)]
/// Vietnamese input method engine for Unix.
struct Args {
    /// whether to run as ibus-daemon
    #[argh(switch)]
    ibus: bool,
    /// print version
    #[argh(switch, short = 'v')]
    version: bool,
}

fn main() {
    let args: Args = argh::from_env();

    match args.ibus {
        true => {
            use tracing_subscriber::layer::SubscriberExt;
            let layer = tracing_journald::Layer::new().unwrap();
            let subscriber = tracing_subscriber::fmt().finish().with(layer);

            match tracing::subscriber::set_global_default(subscriber) {
                Ok(()) => {}
                Err(e) => panic!("cannot init logging to journald: {e}"),
            }
        }
        false => {
            // install global collector configured based on RUST_LOG env var.
            tracing_subscriber::fmt::init();
        }
    }

    if args.version {
        let prog_name = env!("CARGO_PKG_NAME");
        let ver = env!("CARGO_PKG_VERSION");
        eprintln!("{prog_name}-v{ver}");
        return;
    }

    match prepare(args.ibus) {
        Some(bus) => {
            ribus::main(&bus);
            bus.quit();
        }
        None => {
            error!("fail to initialize ibus-gokien");
        }
    }
}

// First find in current executabe dir (for debugging), then
// from `${DATADIR}/ibus/component`.
fn get_engine_xml_path() -> Option<Cow<'static, CStr>> {
    static DEFAULT: &CStr = c"gokien.xml";
    let default = unsafe { OsStr::from_encoded_bytes_unchecked(DEFAULT.to_bytes()) };
    if Path::new(default).is_file() {
        return Some(DEFAULT.into());
    }
    let datadir = option_env!("DATADIR").unwrap_or("/usr/share");
    let mut xml = PathBuf::from(datadir);
    xml.push("ibus/component");
    xml.push(default);
    if PathBuf::from(&xml).is_file() {
        let v = xml.into_os_string().into_encoded_bytes();
        let s = unsafe { CString::from_vec_unchecked(v) };
        return Some(s.into());
    }
    None
}

// Bus shall be alive when ibus_main starting
fn prepare(ibus: bool) -> Option<Bus> {
    let Some(bus) = Bus::new() else {
        error!("cannot connect to ibus deamon");
        return None;
    };
    info!(?bus);
    bus.register_disconnected_signal();
    let Some(file_path) = get_engine_xml_path() else {
        error!("cannot find component file");
        return None;
    };
    let component = Component::from_file(&file_path);
    let component_name = component.get_name();
    info!(?component_name);

    let mut factory = Factory::new(&bus);
    let engines = component.get_engines();
    // let engines = bus.list_engines();
    for e in engines {
        let name = e.get_name();
        info!(engine = ?name);
        factory.add_engine(name, IBusGokienEngine::get_type());
    }

    match ibus {
        false => {
            if !bus.register_component(&component) {
                error!("cannot register component to ibus deamon");
                return None;
            }
        }
        true => {
            let flag = NameFlag::IBUS_BUS_NAME_FLAG_DO_NOT_QUEUE;
            if bus.request_name(component_name, flag).is_none() {
                error!("cannot request {component_name:?} from ibus deamon");
                return None;
            }
        }
    }
    Some(bus)
}
