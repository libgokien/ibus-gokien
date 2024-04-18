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
use tracing::info;

#[derive(FromArgs)]
/// Reach new heights.
struct Args {
    /// whether to run as ibus-daemon
    #[argh(switch)]
    ibus: bool,
    /// print version
    #[argh(switch, short = 'v')]
    version: bool,
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let args: Args = argh::from_env();
    if args.version {
        let prog_name = env!("CARGO_PKG_NAME");
        let ver = env!("CARGO_PKG_VERSION");
        eprintln!("{prog_name}-v{ver}");
        return;
    }
    run(args.ibus);
}

fn get_engine_xml_path() -> Cow<'static, CStr> {
    // first find in current executabe dir, then
    // from `${{DATADIR}}/ibus/component`
    static DEFAULT: &CStr = c"gokien.xml";
    let default = unsafe { OsStr::from_encoded_bytes_unchecked(DEFAULT.to_bytes()) };
    if Path::new(default).is_file() {
        return DEFAULT.into();
    }
    let datadir = option_env!("DATADIR").unwrap_or("/usr/share");
    let mut xml = PathBuf::from(datadir);
    xml.push("ibus/component");
    xml.push(default);
    if PathBuf::from(&xml).is_file() {
        let v = xml.into_os_string().into_encoded_bytes();
        let s = unsafe { CString::from_vec_unchecked(v) };
        return s.into();
    }
    panic!("impossible")
}

fn run(ibus: bool) {
    let Some(bus) = Bus::new() else {
        panic!("cannot connect to ibus deamon");
    };
    info!("bus = {:?}", bus.hello());
    bus.register_disconnected_signal();
    let file_path = get_engine_xml_path();
    let component = Component::new_from_file(&*file_path);
    // component.output();
    let component_name = component.get_name();
    info!(?component_name);

    let mut factory = Factory::new(&bus);
    let engines = component.get_engines();
    // let engines = bus.list_engines();
    for e in engines {
        let name = e.get_name();
        info!("engine = {name:?}");
        factory.add_engine(name, IBusGokienEngine::get_type());
        // let _engine = factory.create_engine(name).unwrap();
    }

    match ibus {
        false => {
            if !bus.register_component(&component) {
                panic!("cannot register component to ibus deamon");
            }
        }
        true => {
            let flag = NameFlag::IBUS_BUS_NAME_FLAG_DO_NOT_QUEUE;
            if bus.request_name(component_name, flag).is_none() {
                panic!("cannot request {component_name:?} from ibus deamon");
            }
        }
    }

    drop(component);
    ribus::main();

    unsafe {
        info!("bus::quit");
        ribus::c::ibus_quit();
    }
}
