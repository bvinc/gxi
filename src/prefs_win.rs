use crate::controller::ControllerRef;
use crate::main_win::MainState;
use crate::rpc::Core;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use log::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PrefsWin {
    controller: ControllerRef,
    window: Window,
    // pub themes: Vec<String>,
    // pub theme_name: String,
    // pub theme: Theme,
    // pub styles: Vec<Style>,
}

impl PrefsWin {
    pub fn new(
        parent: &ApplicationWindow,
        main_state: &Rc<RefCell<MainState>>,
        controller: ControllerRef,
    ) -> Rc<RefCell<PrefsWin>> {
        let glade_src = include_str!("ui/prefs_win.glade");
        let builder = Builder::new_from_string(glade_src);

        let window: Window = builder.get_object("prefs_win").unwrap();
        let font_combo_box: ComboBoxText = builder.get_object("font_combo_box").unwrap();
        let theme_combo_box: ComboBoxText = builder.get_object("theme_combo_box").unwrap();

        {
            let main_state = main_state.borrow();
            for (i, theme_name) in main_state.themes.iter().enumerate() {
                theme_combo_box.append_text(theme_name);
                if &main_state.theme_name == theme_name {
                    debug!("setting active {}", i);
                    theme_combo_box.set_active(Some(i as u32));
                }
            }
        }

        theme_combo_box.connect_changed(
            clone!(@strong controller, @strong main_state => move |cb|{
                if let Some(theme_name) = cb.get_active_text() {
                    debug!("theme changed to {:?}", cb.get_active_text());

                    controller.borrow().set_theme(&theme_name);

                    let mut main_state = main_state.borrow_mut();
                    main_state.theme_name = theme_name.into();
                }
            }),
        );

        let prefs_win = Rc::new(RefCell::new(PrefsWin {
            controller: controller.clone(),
            window: window.clone(),
        }));

        window.set_transient_for(Some(parent));
        window.show_all();

        prefs_win
    }
}
