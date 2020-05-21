use crate::controller::ControllerRef;
use crate::edit_view::EditView;
use crate::prefs_win::PrefsWin;
use crate::proto::{self, ThemeSettings};
use crate::theme::{Color, Style, Theme};
use gio::{ActionMapExt, SimpleAction};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use log::*;
use serde_json::{self, Value};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub struct MainState {
    pub themes: Vec<String>,
    pub theme_name: String,
    pub theme: Theme,
    pub styles: Vec<Style>,
}

pub struct MainWin {
    controller: ControllerRef,
    window: ApplicationWindow,
    notebook: Notebook,
    builder: Builder,
    views: BTreeMap<String, Rc<RefCell<EditView>>>,
    w_to_ev: HashMap<Widget, Rc<RefCell<EditView>>>,
    view_id_to_w: HashMap<String, Widget>,
    state: Rc<RefCell<MainState>>,
}

const GLADE_SRC: &str = include_str!("ui/gxi.glade");

impl MainWin {
    pub fn new(application: &Application, controller: ControllerRef) -> MainWin {
        let glade_src = include_str!("ui/gxi.glade");
        let builder = Builder::new_from_string(glade_src);

        let window: ApplicationWindow = builder.get_object("appwindow").unwrap();
        let notebook: Notebook = builder.get_object("notebook").unwrap();

        let main_win = MainWin {
            controller: controller.clone(),
            window: window.clone(),
            notebook: notebook.clone(),
            builder: builder.clone(),
            views: Default::default(),
            w_to_ev: Default::default(),
            view_id_to_w: Default::default(),
            state: Rc::new(RefCell::new(MainState {
                themes: Default::default(),
                theme_name: "default".to_string(),
                theme: Default::default(),
                styles: Default::default(),
            })),
        };

        window.set_application(Some(application));

        window.connect_delete_event(clone!(@strong window => move |_, _| {
            window.destroy();
            Inhibit(false)
        }));

        {
            let open_action = SimpleAction::new("open", None);
            open_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().handle_open_button();
            }));
            application.add_action(&open_action);
        }
        {
            let new_action = SimpleAction::new("new", None);
            new_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().req_new_view(None);
            }));
            application.add_action(&new_action);
        }
        {
            let prefs_action = SimpleAction::new("prefs", None);
            prefs_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().prefs();
            }));
            application.add_action(&prefs_action);
        }
        {
            let find_action = SimpleAction::new("find", None);
            find_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().find();
            }));
            application.add_action(&find_action);
        }
        {
            let save_action = SimpleAction::new("save", None);
            save_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().handle_save_button();
            }));
            application.add_action(&save_action);
        }
        {
            let save_as_action = SimpleAction::new("save_as", None);
            save_as_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().current_save_as();
            }));
            application.add_action(&save_as_action);
        }
        {
            let close_action = SimpleAction::new("close", None);
            close_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().close();
            }));
            application.add_action(&close_action);
        }
        {
            let close_all_action = SimpleAction::new("close_all", None);
            close_all_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().close_all();
            }));
            application.add_action(&close_all_action);
        }
        {
            let quit_action = SimpleAction::new("quit", None);
            quit_action.connect_activate(clone!(@strong controller => move |_,_| {
                controller.borrow().quit();
            }));
            application.add_action(&quit_action);
        }
        {
            let auto_indent_action =
                SimpleAction::new_stateful("auto_indent", None, &false.to_variant());
            auto_indent_action.connect_change_state(
                clone!(@strong controller => move |action, value| {
                    if value.is_none() {
                        return;
                    }
                    if let Some(value) = value.as_ref() {
                        action.set_state(value);
                        let value: bool = value.get().unwrap();
                        debug!("auto indent {}", value);
                        controller.borrow().set_auto_indent(value)
                    }
                }),
            );
            application.add_action(&auto_indent_action);
        }

        window.show_all();

        main_win
    }
    // pub fn activate(_application: &Application, _shared_queue: Arc<Mutex<SharedQueue>>) {
    //     // TODO
    //     unimplemented!();
    // }
    // pub fn open(_application: &Application, _shared_queue: Arc<Mutex<SharedQueue>>) {
    //     // TODO
    //     unimplemented!();
    // }
}

impl MainWin {
    pub fn available_themes(&mut self, params: &Value) {
        let mut state = self.state.borrow_mut();
        state.themes.clear();
        if let Some(themes) = params["themes"].as_array() {
            for theme in themes {
                if let Some(theme) = theme.as_str() {
                    state.themes.push(theme.to_string());
                }
            }
        }
        if let Some(theme_name) = state.themes.first().map(Clone::clone) {
            state.theme_name = theme_name.clone();
            self.controller.borrow().set_theme(&theme_name);
        }
    }

    pub fn theme_changed(&mut self, params: &Value) {
        let theme_settings = params["theme"].clone();
        let theme_settings: ThemeSettings = match serde_json::from_value(theme_settings) {
            Err(e) => {
                error!("failed to convert theme settings: {}", e);
                return;
            }
            Ok(ts) => ts,
        };

        let selection_foreground = theme_settings
            .selection_foreground
            .map(Color::from_ts_proto);
        let selection = theme_settings.selection.map(Color::from_ts_proto);

        let theme = Theme::from_proto(&theme_settings);
        {
            let mut state = self.state.borrow_mut();
            state.theme = theme;
        }

        let selection_sytle = Style {
            fg_color: selection_foreground,
            bg_color: selection,
            weight: None,
            italic: None,
            underline: None,
        };

        self.set_style(0, selection_sytle);
    }

    pub fn available_plugins(&mut self, params: &Value) {
        error!("UNHANDLED available_plugins {}", params);
    }

    pub fn config_changed(&mut self, params: &Value) {
        let view_id = {
            let view_id = params["view_id"].as_str();
            if view_id.is_none() {
                return;
            }
            view_id.unwrap().to_string()
        };

        if let Some(ev) = self.views.get(&view_id) {
            ev.borrow_mut().config_changed(&params["changes"]);
        }
    }

    pub fn find_status(&mut self, params: &Value) {
        let view_id = {
            let view_id = params["view_id"].as_str();
            if view_id.is_none() {
                return;
            }
            view_id.unwrap().to_string()
        };

        if let Some(ev) = self.views.get(&view_id) {
            ev.borrow_mut().find_status(&params["queries"]);
        }
    }

    pub fn def_style(&mut self, params: &Value) {
        let style: proto::Style = serde_json::from_value(params.clone()).unwrap();
        let style = Style::from_proto(&style);

        if let Some(id) = params["id"].as_u64() {
            let id = id as usize;

            self.set_style(id, style);
        }
    }

    pub fn set_style(&self, id: usize, style: Style) {
        let mut state = self.state.borrow_mut();
        // bump the array size up if needed
        while state.styles.len() < id {
            state.styles.push(Style {
                fg_color: None,
                bg_color: None,
                weight: None,
                italic: None,
                underline: None,
            })
        }
        if state.styles.len() == id {
            state.styles.push(style);
        } else {
            state.styles[id] = style;
        }
    }

    pub fn update(&mut self, params: &Value) {
        trace!("handling update {:?}", params);

        let view_id = {
            let view_id = params["view_id"].as_str();
            if view_id.is_none() {
                return;
            }
            view_id.unwrap().to_string()
        };

        if let Some(ev) = self.views.get(&view_id) {
            EditView::update(&ev, params);
        }
    }

    pub fn scroll_to(&mut self, params: &Value) {
        trace!("handling scroll_to {:?}", params);
        let view_id = {
            let view_id = params["view_id"].as_str();
            if view_id.is_none() {
                return;
            }
            view_id.unwrap().to_string()
        };

        let line = {
            match params["line"].as_u64() {
                None => return,
                Some(line) => line,
            }
        };

        let col = {
            match params["col"].as_u64() {
                None => return,
                Some(col) => col,
            }
        };

        match self.views.get(&view_id) {
            None => debug!("failed to find view {}", view_id),
            Some(edit_view) => {
                let idx = self.notebook.page_num(&edit_view.borrow().root_widget);
                self.notebook.set_current_page(idx);
                EditView::scroll_to(edit_view, line, col);
            }
        }
    }

    /// Display the FileChooserDialog for opening, send the result to the Xi core.
    /// This may call the GTK main loop.  There must not be any RefCell borrows out while this
    /// function runs.
    pub fn handle_open_button(&self) {
        let parent: Option<&ApplicationWindow> = None;
        let fcd = FileChooserDialog::new(None, parent, FileChooserAction::Open);
        // fcd.set_transient_for(Some(&main_win.window.clone()));
        fcd.add_button("Open", ResponseType::Other(33));
        fcd.set_default_response(ResponseType::Other(33));
        fcd.set_select_multiple(true);
        let response = fcd.run(); // Can call main loop, can't have any borrows out
        debug!("open response = {}", response);
        if response == ResponseType::Other(33) {
            for file in fcd.get_filenames() {
                self.req_new_view(Some(&file.to_string_lossy()));
            }
        }
        fcd.destroy();
    }

    pub fn handle_save_button(&self) {
        let edit_view = self.get_current_edit_view().clone();
        if edit_view.borrow().file_name.is_some() {
            let ev = edit_view.borrow_mut();
            self.controller
                .borrow()
                .core()
                .save(&ev.view_id, ev.file_name.as_ref().unwrap());
        } else {
            self.save_as(&edit_view);
        }
    }

    pub fn current_save_as(&self) {
        let edit_view = self.get_current_edit_view().clone();
        self.save_as(&edit_view);
    }

    /// Display the FileChooserDialog, send the result to the Xi core.
    /// This may call the GTK main loop.  There must not be any RefCell borrows out while this
    /// function runs.
    pub fn save_as(&self, edit_view: &Rc<RefCell<EditView>>) {
        let parent: Option<&ApplicationWindow> = None;
        let fcd = FileChooserDialog::new(None, parent, FileChooserAction::Save);
        // fcd.set_transient_for(Some(&main_win.borrow().window.clone()));
        fcd.add_button("Save", ResponseType::Other(33));
        fcd.set_default_response(ResponseType::Other(33));
        let response = fcd.run(); // Can call main loop, can't have any borrows out
        debug!("save response = {}", response);
        if response == ResponseType::Other(33) {
            // let win = main_win;
            if let Some(file) = fcd.get_filename() {
                debug!("saving {:?}", file);
                let view_id = edit_view.borrow().view_id.clone();
                let file = file.to_string_lossy();
                self.controller.borrow().core().save(&view_id, &file);
                edit_view.borrow_mut().set_file(&file);
            }
        }
        fcd.destroy();
    }

    pub fn prefs(&self) {
        // let (main_state, core) = {
        //     let main_win = main_win.borrow();
        //     (main_win.state.clone(), main_win.core.clone())
        // };
        let main_state = self.state.clone();
        let controller = self.controller.clone();
        let prefs_win = PrefsWin::new(&self.window, &main_state, controller.clone());
        // prefs_win.run();
    }

    pub fn find(&self) {
        let edit_view = self.get_current_edit_view().clone();
        edit_view.borrow().start_search();
    }

    fn get_current_edit_view(&self) -> Rc<RefCell<EditView>> {
        if let Some(idx) = self.notebook.get_current_page() {
            if let Some(w) = self.notebook.get_nth_page(Some(idx)) {
                if let Some(edit_view) = self.w_to_ev.get(&w) {
                    return edit_view.clone();
                }
            }
        }
        unreachable!("failed to get the current editview");
    }

    fn req_new_view(&self, file_name: Option<&str>) {
        self.controller.borrow().req_new_view(file_name)
    }

    pub fn new_view_response(&mut self, file_name: Option<String>, value: &Value) {
        let controller = self.controller.clone();
        if let Some(view_id) = value.as_str() {
            let edit_view = EditView::new(
                self.state.clone(),
                self.controller.clone(),
                file_name,
                view_id,
            );
            {
                let ev = edit_view.borrow();
                let page_num =
                    self.notebook
                        .insert_page(&ev.root_widget, Some(&ev.tab_widget), None);
                if let Some(w) = self.notebook.get_nth_page(Some(page_num)) {
                    self.w_to_ev.insert(w.clone(), edit_view.clone());
                    self.view_id_to_w.insert(view_id.to_string(), w);
                }

                let vid = view_id.to_string();
                ev.close_button.connect_clicked(
                    clone!(@strong controller, @strong vid => move |_| {
                        controller.borrow().close_view(&vid);
                    }),
                );
            }

            self.views.insert(view_id.to_string(), edit_view);
        }
    }

    pub fn close_all(&mut self) {
        let edit_view = self.get_current_edit_view();
        self.close_view(&edit_view.borrow().view_id);
    }

    pub fn close(&mut self) {
        let edit_view = self.get_current_edit_view();
        self.close_view(&edit_view.borrow().view_id);
    }

    pub fn close_view(&mut self, view_id: &str) {
        if let Some(w) = self.view_id_to_w.get(view_id).map(Clone::clone) {
            let edit_view = self.w_to_ev.get(&w).expect("ev not found");
            let pristine = edit_view.borrow().pristine;
            if !pristine {
                let builder = Builder::new_from_string(&GLADE_SRC);
                let ask_save_dialog: Dialog = builder.get_object("ask_save_dialog").unwrap();
                let ret = ask_save_dialog.run();
                ask_save_dialog.destroy();
                debug!("ask_save_dialog = {}", ret);
                match ret {
                    ResponseType::Other(1) => self.save_as(edit_view),
                    ResponseType::Other(2) => return,
                    _ => {}
                };
            }

            if let Some(page_num) = self.notebook.page_num(&w) {
                self.notebook.remove_page(Some(page_num));
            }
            self.w_to_ev.remove(&w.clone());
        }
        self.view_id_to_w.remove(view_id);
    }

    pub fn quit(&self) {
        self.window.destroy();
    }
}
