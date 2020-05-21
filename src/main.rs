#![recursion_limit = "128"]

mod channel;
mod clipboard;
mod controller;
mod edit_view;
mod linecache;
mod main_win;
mod prefs_win;
mod proto;
mod rpc;
mod scrollable_drawing_area;
mod theme;
mod xi_thread;

use crate::channel::Sender;
use crate::controller::{Controller, CoreMsg};
use clap::{Arg, SubCommand};
use gio::prelude::*;
use gio::{ApplicationExt, ApplicationFlags, FileExt};
use glib::clone;
use gtk::{Application};
use log::*;
use main_win::MainWin;
use rpc::{Core, Handler};
use serde_json::Value;
use std::any::Any;
use std::cell::RefCell;
use std::env::args;
use dirs_next::home_dir;

// pub struct SharedQueue {
//     queue: VecDeque<CoreMsg>,
// }

// impl SharedQueue {
//     pub fn add_core_msg(&mut self, msg: CoreMsg) {
//         if self.queue.is_empty() {
//             self.pipe_writer
//                 .write_all(&[0u8])
//                 .expect("failed to write to signalling pipe");
//         }
//         trace!("pushing to queue");
//         self.queue.push_back(msg);
//     }
// }

trait IdleCallback: Send {
    fn call(self: Box<Self>, a: &Any);
}

impl<F: FnOnce(&Any) + Send> IdleCallback for F {
    fn call(self: Box<F>, a: &Any) {
        (*self)(a)
    }
}

// struct QueueSource {
//     win: Rc<RefCell<MainWin>>,
//     sender: Sender<CoreMsg>,
// }

// impl SourceFuncs for QueueSource {
//     fn check(&self) -> bool {
//         false
//     }

//     fn prepare(&self) -> (bool, Option<u32>) {
//         (false, None)
//     }

//     fn dispatch(&self) -> bool {
//         trace!("dispatch");
//         let mut shared_queue = self.queue.lock().unwrap();
//         while let Some(msg) = shared_queue.queue.pop_front() {
//             trace!("found a msg");
//             MainWin::handle_msg(self.win.clone(), msg);
//         }
//         let mut buf = [0u8; 64];
//         shared_queue
//             .pipe_reader
//             .try_read(&mut buf)
//             .expect("failed to read signalling pipe");
//         true
//     }
// }

#[derive(Clone)]
struct MyHandler {
    sender: Sender<CoreMsg>,
}

impl MyHandler {
    fn new(sender: Sender<CoreMsg>) -> MyHandler {
        MyHandler { sender }
    }
}

impl Handler for MyHandler {
    fn notification(&self, method: &str, params: &Value) {
        debug!(
            "CORE --> {{\"method\": \"{}\", \"params\":{}}}",
            method, params
        );
        let method2 = method.to_string();
        let params2 = params.clone();
        self.sender.send(CoreMsg::Notification {
            method: method2,
            params: params2,
        });
    }
}

fn main() {
    env_logger::init();
    // let matches = App::new("gxi")
    //     .version("0.2.0")
    //     .author("brainn <brainn@gmail.com>")
    //     .about("Xi frontend")
    //     .arg(Arg::with_name("FILE")
    //         .multiple(true)
    //         .help("file to open")
    //     )
    //     .get_matches();

    // let mut files = vec![];
    // if matches.is_present("FILE") {
    //     files = matches.values_of("FILE").unwrap().collect::<Vec<_>>();
    // }
    // debug!("files {:?}", files);

    let controller = Controller::new();
    let controller2 = controller.clone();
    let (chan, sender) = channel::Channel::new(move |msg| {
        controller2.borrow().handle_msg(msg);
    });
    controller.borrow_mut().set_sender(sender.clone());
    controller.borrow_mut().set_channel(chan);

    // let queue: VecDeque<CoreMsg> = Default::default();
    // let (reader, writer) = pipe().unwrap();
    // let reader_raw_fd = reader.as_raw_fd();

    // let shared_queue = Arc::new(Mutex::new(SharedQueue {
    //     queue: queue.clone(),
    //     pipe_writer: writer,
    //     pipe_reader: reader,
    // }));

    let (xi_peer, rx) = xi_thread::start_xi_thread();
    let handler = MyHandler::new(sender.clone());
    let core = Core::new(xi_peer, rx, handler.clone());
    controller.borrow_mut().set_core(core);

    let application =
        Application::new(Some("com.github.bvinc.gxi"), ApplicationFlags::HANDLES_OPEN)
            .expect("failed to create gtk application");

    let mut config_dir = None;
    let mut plugin_dir = None;
    if let Some(home_dir) = home_dir() {
        let xi_config = home_dir.join(".config").join("xi");
        let xi_plugin = xi_config.join("plugins");
        config_dir = xi_config.to_str().map(|s| s.to_string());
        plugin_dir = xi_plugin.to_str().map(|s| s.to_string());
    }

    application.connect_startup(clone!(@strong controller => move |application| {
        debug!("startup");
        controller.borrow().core().client_started(config_dir.clone(), plugin_dir.clone());

        let main_win = MainWin::new(application, controller.clone());
        controller.borrow_mut().set_main_win(main_win);

        // let source = new_source(QueueSource {
        //     win: main_win.clone(),
        //     sender: sender.clone(),
        // });
        // unsafe {
        //     use glib::translate::ToGlibPtr;
        //     ::glib_sys::g_source_add_unix_fd(source.to_glib_none().0, reader_raw_fd, ::glib_sys::G_IO_IN);
        // }
        // let main_context = MainContext::default();
        // source.attach(Some(&main_context));
    }));

    application.connect_activate(clone!(@strong controller => move |application| {
        debug!("activate");

        controller.borrow().req_new_view(None);
    }));

    application.connect_open(clone!(@strong controller => move |_,files,s| {
        debug!("open");

        for file in files {
            let path = file.get_path();
            if path.is_none() { continue; }
            let path = path.unwrap();
            let path = path.to_string_lossy().into_owned();

            controller.borrow().req_new_view(Some(&path));
        }
    }));
    application.connect_shutdown(move |_| {
        debug!("shutdown");
    });

    application.run(&args().collect::<Vec<_>>());
}
