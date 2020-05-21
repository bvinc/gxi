use crate::channel::{Channel, Sender};
use crate::main_win::MainWin;
use crate::rpc::Core;
use log::*;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum CoreMsg {
    Notification {
        method: String,
        params: Value,
    },
    NewViewReply {
        file_name: Option<String>,
        value: Value,
    },
}

pub type ControllerRef = Rc<RefCell<Controller>>;

#[derive(Default)]
pub struct Controller {
    core: Option<Core>,
    channel: Option<Channel<CoreMsg>>,
    sender: Option<Sender<CoreMsg>>,
    main_win: Option<RefCell<MainWin>>,
}

impl Controller {
    pub fn new() -> ControllerRef {
        Rc::new(RefCell::new(Controller {
            ..Default::default()
        }))
    }

    pub fn core(&self) -> &Core {
        self.core.as_ref().expect("no core")
    }

    pub fn set_core(&mut self, core: Core) {
        self.core = Some(core);
    }

    pub fn set_channel(&mut self, channel: Channel<CoreMsg>) {
        self.channel = Some(channel);
    }

    pub fn sender(&self) -> &Sender<CoreMsg> {
        self.sender.as_ref().expect("no sender")
    }
    pub fn set_sender(&mut self, sender: Sender<CoreMsg>) {
        self.sender = Some(sender);
    }

    fn main_win(&self) -> &RefCell<MainWin> {
        self.main_win.as_ref().expect("no main win")
    }
    pub fn set_main_win(&mut self, main_win: MainWin) {
        self.main_win = Some(RefCell::new(main_win));
    }

    pub fn handle_msg(&self, msg: CoreMsg) {
        match msg {
            CoreMsg::NewViewReply { file_name, value } => self
                .main_win()
                .borrow_mut()
                .new_view_response(file_name, &value),
            CoreMsg::Notification { method, params } => {
                match method.as_ref() {
                    "available_themes" => self.main_win().borrow_mut().available_themes(&params),
                    "available_plugins" => self.main_win().borrow_mut().available_plugins(&params),
                    "config_changed" => self.main_win().borrow_mut().config_changed(&params),
                    "def_style" => self.main_win().borrow_mut().def_style(&params),
                    "find_status" => self.main_win().borrow_mut().find_status(&params),
                    "update" => self.main_win().borrow_mut().update(&params),
                    "scroll_to" => self.main_win().borrow_mut().scroll_to(&params),
                    "theme_changed" => self.main_win().borrow_mut().theme_changed(&params),
                    _ => {
                        error!("!!! UNHANDLED NOTIFICATION: {}", method);
                    }
                };
            }
        };
    }

    pub fn req_new_view(&self, file_name: Option<&str>) {
        let mut params = json!({});
        if let Some(file_name) = file_name {
            params["file_path"] = json!(file_name);
        } else {
            params["file_path"] = Value::Null;
        }

        let sender2 = self.sender().clone();
        let file_name2 = file_name.map(|s| s.to_string());
        self.core().send_request("new_view", &params, move |value| {
            let value = value.clone();
            sender2
                .send(CoreMsg::NewViewReply {
                    file_name: file_name2,
                    value,
                })
                .expect("send failed");
        });
    }
    pub fn set_theme(&self, theme_name: &str) {
        self.core()
            .send_notification("set_theme", &json!({ "theme_name": theme_name }));
    }

    pub fn set_auto_indent(&self, auto_indent: bool) {
        self.core()
            .modify_user_config(&json!("general"), &json!({ "auto_indent": auto_indent }));
    }

    pub fn save(&self, view_id: &str, file_path: &str) {
        self.core().save(view_id, file_path)
    }
    pub fn close_view(&self, view_id: &str) {
        self.main_win().borrow_mut().close_view(view_id);
        self.core().close_view(view_id)
    }

    pub fn handle_open_button(&self) {
        self.main_win().borrow_mut().handle_open_button();
    }
    pub fn prefs(&self) {
        self.main_win().borrow_mut().prefs();
    }
    pub fn find(&self) {
        self.main_win().borrow_mut().find();
    }
    pub fn handle_save_button(&self) {
        self.main_win().borrow_mut().handle_save_button();
    }
    pub fn current_save_as(&self) {
        self.main_win().borrow_mut().current_save_as();
    }
    pub fn close(&self) {
        self.main_win().borrow_mut().close();
    }
    pub fn close_all(&self) {
        self.main_win().borrow_mut().close_all();
    }
    pub fn quit(&self) {
        self.main_win().borrow_mut().quit();
    }
}
