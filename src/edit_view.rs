use crate::controller::ControllerRef;
use crate::linecache::{Line, LineCache};
use crate::main_win::MainState;
use crate::rpc::{self};
use crate::scrollable_drawing_area::ScrollableDrawingArea;
use crate::theme::set_source_color;
use cairo::Context;
use gdk::enums::key;
use gdk::*;
use glib::clone;
use gtk::prelude::*;
use gtk::{self, *};
use log::*;
use pango::{self, *};
use pangocairo::functions::*;
use serde_json::Value;
use std::cell::RefCell;
use std::cmp::{max, min};
use std::ops::Range;
use std::rc::Rc;
use std::u32;

pub struct EditView {
    controller: ControllerRef,
    main_state: Rc<RefCell<MainState>>,
    pub view_id: String,
    pub file_name: Option<String>,
    pub pristine: bool,
    pub line_da: ScrollableDrawingArea,
    pub da: ScrollableDrawingArea,
    pub root_widget: gtk::Box,
    pub tab_widget: gtk::Box,
    pub label: Label,
    pub close_button: Button,
    search_bar: SearchBar,
    search_entry: SearchEntry,
    replace_expander: Expander,
    replace_revealer: Revealer,
    replace_entry: Entry,
    find_status_label: Label,
    hadj: Adjustment,
    vadj: Adjustment,
    line_cache: LineCache,
    font_height: f64,
    font_width: f64,
    font_ascent: f64,
    font_descent: f64,
    font_desc: FontDescription,
    visible_lines: Range<u64>,
}

impl EditView {
    pub fn new(
        main_state: Rc<RefCell<MainState>>,
        controller: ControllerRef,
        file_name: Option<String>,
        view_id: &str,
    ) -> Rc<RefCell<EditView>> {
        // let da = DrawingArea::new();
        let da = ScrollableDrawingArea::new();
        let line_da = ScrollableDrawingArea::new();
        line_da.set_size_request(100, 100);
        let sw_hadj: Option<&Adjustment> = None;
        let sw_vadj: Option<&Adjustment> = None;
        let scrolled_window = ScrolledWindow::new(sw_hadj, sw_vadj);
        scrolled_window.add(&da);

        let hadj = Adjustment::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let vadj = Adjustment::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        vadj.set_step_increment(1.0);

        scrolled_window.set_hadjustment(Some(&hadj));
        scrolled_window.set_vadjustment(Some(&vadj));
        scrolled_window.set_kinetic_scrolling(true);

        da.set_events(
            EventMask::BUTTON_PRESS_MASK
                | EventMask::BUTTON_RELEASE_MASK
                | EventMask::BUTTON_MOTION_MASK
                | EventMask::SCROLL_MASK
                | EventMask::SMOOTH_SCROLL_MASK,
        );
        debug!("events={:?}", da.get_events());
        da.set_can_focus(true);

        let find_rep_src = include_str!("ui/find_replace.glade");
        let find_rep_builder = Builder::new_from_string(find_rep_src);
        let search_bar: SearchBar = find_rep_builder.get_object("search_bar").unwrap();
        let replace_expander: Expander = find_rep_builder.get_object("replace_expander").unwrap();
        let replace_revealer: Revealer = find_rep_builder.get_object("replace_revealer").unwrap();
        let replace_entry: Entry = find_rep_builder.get_object("replace_entry").unwrap();
        let replace_button: Button = find_rep_builder.get_object("replace_button").unwrap();
        let replace_all_button: Button = find_rep_builder.get_object("replace_all_button").unwrap();
        let find_status_label: Label = find_rep_builder.get_object("find_status_label").unwrap();

        // let overlay: Overlay = frame_builder.get_object("overlay").unwrap();
        // let search_revealer: Revealer = frame_builder.get_object("revealer").unwrap();
        // let frame: Frame = frame_builder.get_object("frame").unwrap();
        let search_entry: SearchEntry = find_rep_builder.get_object("search_entry").unwrap();
        let go_down_button: Button = find_rep_builder.get_object("go_down_button").unwrap();
        let go_up_button: Button = find_rep_builder.get_object("go_up_button").unwrap();

        // let style_context = frame.get_style_context().unwrap();
        // style_context.add_provider(&css_provider, 1);

        let line_hbox = Box::new(Orientation::Horizontal, 0);
        line_hbox.pack_start(&line_da, false, false, 0);
        line_hbox.pack_start(&scrolled_window, true, true, 0);

        let main_vbox = Box::new(Orientation::Vertical, 0);
        main_vbox.pack_start(&search_bar, false, false, 0);
        main_vbox.pack_start(&line_hbox, true, true, 0);

        main_vbox.show_all();

        // Make the widgets for the tab
        let tab_hbox = gtk::Box::new(Orientation::Horizontal, 5);
        let label = Label::new(Some(""));
        tab_hbox.add(&label);
        let close_button = Button::new_from_icon_name(Some("window-close"), IconSize::SmallToolbar);
        tab_hbox.add(&close_button);
        tab_hbox.show_all();

        use ::fontconfig::fontconfig;
        use std::ffi::CString;
        unsafe {
            let fonts_dir = CString::new("fonts").unwrap();
            let ret = fontconfig::FcConfigAppFontAddDir(
                fontconfig::FcConfigGetCurrent(),
                fonts_dir.as_ptr() as *const u8,
            );
            debug!("fc ret = {}", ret);
        }

        let pango_ctx = da.get_pango_context().expect("failed to get pango ctx");
        for family in pango_ctx.list_families() {
            if !family.is_monospace() {
                continue;
            }
            debug!(
                "font family {:?} monospace: {}",
                family.get_name(),
                family.is_monospace()
            );
        }
        let font_desc = FontDescription::from_string("Inconsolata 14");
        pango_ctx.set_font_description(&font_desc);
        let language = pango_ctx
            .get_language()
            .expect("failed to get pango language");
        let fontset = pango_ctx
            .load_fontset(&font_desc, &language)
            .expect("failed to load font set");
        let metrics = fontset.get_metrics().expect("failed to load font metrics");

        // cr.select_font_face("Inconsolata", ::cairo::enums::FontSlant::Normal, ::cairo::enums::FontWeight::Normal);
        // cr.set_font_size(16.0);
        // let font_extents = cr.font_extents();

        let layout = pango::Layout::new(&pango_ctx);
        layout.set_text("a");
        let (_, log_extents) = layout.get_extents();
        debug!("size: {:?}", log_extents);

        let font_height = f64::from(log_extents.height) / f64::from(pango::SCALE);
        let font_width = f64::from(log_extents.width) / f64::from(pango::SCALE);
        let font_ascent = f64::from(metrics.get_ascent()) / f64::from(pango::SCALE);
        let font_descent = f64::from(metrics.get_descent()) / f64::from(pango::SCALE);

        debug!(
            "font metrics: {} {} {} {}",
            font_width, font_height, font_ascent, font_descent
        );

        let edit_view = Rc::new(RefCell::new(EditView {
            controller: controller.clone(),
            main_state: main_state.clone(),
            file_name,
            pristine: true,
            view_id: view_id.to_string(),
            line_da: line_da.clone(),
            da: da.clone(),
            root_widget: main_vbox.clone(),
            tab_widget: tab_hbox.clone(),
            label: label.clone(),
            close_button: close_button.clone(),
            search_bar: search_bar.clone(),
            search_entry: search_entry.clone(),
            replace_expander: replace_expander.clone(),
            replace_revealer: replace_revealer.clone(),
            replace_entry: replace_entry.clone(),
            find_status_label: find_status_label.clone(),
            hadj: hadj.clone(),
            vadj: vadj.clone(),
            line_cache: LineCache::new(),
            font_height,
            font_width,
            font_ascent,
            font_descent,
            font_desc,
            visible_lines: 0..1,
        }));

        edit_view.borrow_mut().update_title();

        line_da.connect_draw(clone!(@strong edit_view => move |_,ctx| {
            edit_view.borrow_mut().handle_line_draw(&ctx)
        }));

        da.connect_button_press_event(clone!(@strong edit_view => move |_,eb| {
            edit_view.borrow().handle_button_press(eb)
        }));

        da.connect_draw(clone!(@strong edit_view => move |_,ctx| {
            edit_view.borrow_mut().handle_draw(&ctx)
        }));

        da.connect_key_press_event(clone!(@strong edit_view => move |_, ek| {
            edit_view.borrow_mut().handle_key_press_event(ek)
        }));

        da.connect_motion_notify_event(clone!(@strong edit_view => move |_,em| {
            edit_view.borrow_mut().handle_drag(em)
        }));

        da.connect_realize(|w| {
            // Set the text cursor
            if let Some(disp) = DisplayManager::get().get_default_display() {
                let cur = Cursor::new_for_display(&disp, CursorType::Xterm);
                if let Some(win) = w.get_window() {
                    win.set_cursor(Some(&cur))
                }
            }
            w.grab_focus();
        });

        da.connect_scroll_event(clone!(@strong edit_view => move |_,es| {
            edit_view.borrow_mut().handle_scroll(es)
        }));

        da.connect_size_allocate(clone!(@strong edit_view => move |_,alloc| {
            debug!("Size changed to w={} h={}", alloc.width, alloc.height);
            edit_view.borrow_mut().da_size_allocate(alloc.width, alloc.height);
        }));

        search_entry.connect_search_changed(clone!(@strong edit_view => move |w| {
            edit_view.borrow_mut().search_changed(w.get_text().map(|gs| gs.as_str().to_owned()));
        }));

        search_entry.connect_activate(clone!(@strong edit_view => move |w| {
            edit_view.borrow_mut().find_next();
        }));

        search_entry.connect_stop_search(clone!(@strong edit_view => move |w| {
            edit_view.borrow().stop_search();
        }));

        replace_expander.connect_property_expanded_notify(
            clone!(@strong replace_revealer => move|w| {
                if w.get_expanded() {
                    replace_revealer.set_reveal_child(true);
                } else {
                    replace_revealer.set_reveal_child(false);
                }
            }),
        );

        replace_button.connect_clicked(clone!(@strong edit_view => move |w| {
            edit_view.borrow().replace();
        }));

        replace_all_button.connect_clicked(clone!(@strong edit_view => move |w| {
            edit_view.borrow().replace_all();
        }));

        go_down_button.connect_clicked(clone!(@strong edit_view => move |_| {
            edit_view.borrow_mut().find_next();
        }));

        go_up_button.connect_clicked(clone!(@strong edit_view => move |_| {
            edit_view.borrow_mut().find_prev();
        }));

        vadj.connect_value_changed(clone!(@strong edit_view => move |_| {
            edit_view.borrow_mut().update_visible_scroll_region();
        }));

        edit_view
    }
}

fn convert_gtk_modifier(mt: ModifierType) -> u32 {
    let mut ret = 0;
    if mt.contains(ModifierType::SHIFT_MASK) {
        ret |= rpc::XI_SHIFT_KEY_MASK;
    }
    if mt.contains(ModifierType::CONTROL_MASK) {
        ret |= rpc::XI_CONTROL_KEY_MASK;
    }
    if mt.contains(ModifierType::MOD1_MASK) {
        ret |= rpc::XI_ALT_KEY_MASK;
    }
    ret
}

impl EditView {
    pub fn set_file(&mut self, file_name: &str) {
        self.file_name = Some(file_name.to_string());
        self.update_title();
    }

    fn update_title(&self) {
        let title = match self.file_name {
            Some(ref f) => f
                .split(::std::path::MAIN_SEPARATOR)
                .last()
                .unwrap_or("Untitled")
                .to_string(),
            None => "Untitled".to_string(),
        };

        let mut full_title = String::new();
        if !self.pristine {
            full_title.push('*');
        }
        full_title.push_str(&title);

        trace!("setting title to {}", full_title);
        self.label.set_text(&full_title);
    }

    pub fn config_changed(&mut self, changes: &Value) {
        if let Some(map) = changes.as_object() {
            for (name, value) in map {
                match name.as_ref() {
                    "font_size" => {
                        if let Some(font_size) = value.as_u64() {
                            self.font_desc.set_size(font_size as i32 * pango::SCALE);
                        }
                    }
                    "font_face" => {
                        if let Some(font_face) = value.as_str() {
                            if font_face == "InconsolataGo" {
                                // TODO This shouldn't be necessary, but the only font I've found
                                // to bundle is "Inconsolata"
                                self.font_desc.set_family("Inconsolata");
                            } else {
                                self.font_desc.set_family(font_face);
                            }
                        }
                    }
                    _ => {
                        error!("unhandled config option {}", name);
                    }
                }
            }
        }
    }

    pub fn update(edit_view: &Rc<RefCell<EditView>>, params: &Value) {
        let update = &params["update"];
        let (text_width, text_height, vadj, hadj) = {
            let mut ev = edit_view.borrow_mut();
            ev.line_cache.apply_update(update);

            if let Some(pristine) = update["pristine"].as_bool() {
                if ev.pristine != pristine {
                    ev.pristine = pristine;
                    ev.update_title();
                }
            }

            ev.line_da.queue_draw();
            ev.da.queue_draw();

            let (text_width, text_height) = ev.get_text_size();
            let vadj = ev.vadj.clone();
            let hadj = ev.hadj.clone();

            (text_width, text_height, vadj, hadj)
        };

        // update scrollbars to the new text width and height
        vadj.set_lower(0f64);
        vadj.set_upper(text_height as f64);
        if vadj.get_value() + vadj.get_page_size() > vadj.get_upper() {
            vadj.set_value(vadj.get_upper() - vadj.get_page_size())
        }

        // hadj.set_lower(0f64);
        // hadj.set_upper(text_width as f64);
        // if hadj.get_value() + hadj.get_page_size() > hadj.get_upper() {
        //     hadj.set_value(hadj.get_upper() - hadj.get_page_size())
        // }
    }

    pub fn da_px_to_cell(&self, main_state: &MainState, x: f64, y: f64) -> (u64, u64) {
        // let first_line = (vadj.get_value() / font_extents.height) as usize;
        let x = x + self.hadj.get_value();
        let y = y + self.vadj.get_value();

        let mut y = y - self.font_descent;
        if y < 0.0 {
            y = 0.0;
        }
        let line_num = (y / self.font_height) as u64;
        let index = if let Some(line) = self.line_cache.get_line(line_num) {
            let pango_ctx = self
                .da
                .get_pango_context()
                .expect("failed to get pango ctx");

            let layout = self.create_layout_for_line(&pango_ctx, &main_state, line);
            let (_, index, trailing) = layout.xy_to_index(x as i32 * pango::SCALE, 0);
            index + trailing
        } else {
            0
        };
        (index as u64, (y / self.font_height) as u64)
    }

    fn da_size_allocate(&mut self, da_width: i32, da_height: i32) {
        debug!("DA SIZE ALLOCATE");
        let vadj = self.vadj.clone();
        vadj.set_page_size(f64::from(da_height));
        let hadj = self.hadj.clone();
        hadj.set_page_size(f64::from(da_width));

        self.update_visible_scroll_region();
    }

    /// Inform core that the visible scroll region has changed
    fn update_visible_scroll_region(&mut self) {
        let main_state = self.main_state.borrow();
        let da_height = self.da.get_allocated_height();
        let (_, first_line) = self.da_px_to_cell(&main_state, 0.0, 0.0);
        let (_, last_line) = self.da_px_to_cell(&main_state, 0.0, f64::from(da_height));
        let last_line = last_line + 1;
        let visible_lines = first_line..last_line;
        if visible_lines != self.visible_lines {
            self.visible_lines = visible_lines;
            self.controller
                .borrow()
                .core()
                .scroll(&self.view_id, first_line, last_line);
        }
    }

    fn get_text_size(&self) -> (f64, f64) {
        let da_width = f64::from(self.da.get_allocated_width());
        let da_height = f64::from(self.da.get_allocated_height());
        let num_lines = self.line_cache.height();

        let all_text_height = num_lines as f64 * self.font_height + self.font_descent;
        let height = if da_height > all_text_height {
            da_height
        } else {
            all_text_height
        };

        let all_text_width = self.line_cache.width() as f64 * self.font_width;
        let width = if da_width > all_text_width {
            da_width
        } else {
            all_text_width
        };
        (width, height)
    }

    pub fn handle_line_draw(&mut self, cr: &Context) -> Inhibit {
        // let foreground = self.main_state.borrow().theme.foreground;
        let theme = &self.main_state.borrow().theme;

        let da_width = self.line_da.get_allocated_width();
        let da_height = self.line_da.get_allocated_height();

        let num_lines = self.line_cache.height();

        let vadj = self.vadj.clone();
        // let hadj = self.hadj.clone();
        trace!("drawing.  vadj={}, {}", vadj.get_value(), vadj.get_upper());

        let first_line = (vadj.get_value() / self.font_height) as u64;
        let last_line = ((vadj.get_value() + f64::from(da_height)) / self.font_height) as u64 + 1;
        let last_line = min(last_line, num_lines);

        // Find missing lines
        let mut found_missing = false;
        for i in first_line..last_line {
            if self.line_cache.get_line(i).is_none() {
                debug!("missing line {}", i);
                found_missing = true;
            }
        }

        // We've already missed our chance to draw these lines, but we need to request them for the
        // next frame.  This needs to be improved to prevent flashing.
        if found_missing {
            debug!(
                "didn't have some lines, requesting, lines {}-{}",
                first_line, last_line
            );
            self.controller.borrow().core().request_lines(
                &self.view_id,
                first_line as u64,
                last_line as u64,
            );
        }

        let pango_ctx = self.da.get_pango_context().unwrap();
        pango_ctx.set_font_description(&self.font_desc);

        // Calculate ordinal or max line length
        let padding: usize = format!("{}", num_lines.saturating_sub(1)).len();

        let main_state = self.main_state.borrow();

        // Just get the gutter size
        let mut gutter_size = 0.0;
        let pango_ctx = self
            .da
            .get_pango_context()
            .expect("failed to get pango ctx");
        let linecount_layout =
            self.create_layout_for_linecount(&pango_ctx, &main_state, 0, padding);
        update_layout(cr, &linecount_layout);
        // show_layout(cr, &linecount_layout);

        let linecount_offset = (linecount_layout.get_extents().1.width / pango::SCALE) as f64;
        if linecount_offset > gutter_size {
            gutter_size = linecount_offset;
        }
        let gutter_size = gutter_size as i32;

        self.line_da.set_size_request(gutter_size, 0);

        // Draw the gutter background
        set_source_color(cr, theme.gutter);
        cr.rectangle(0.0, 0.0, f64::from(da_width), f64::from(da_height));
        cr.fill();

        for i in first_line..last_line {
            // Keep track of the starting x position
            if let Some(_) = self.line_cache.get_line(i) {
                cr.move_to(0.0, self.font_height * (i as f64) - vadj.get_value());

                set_source_color(cr, theme.gutter_foreground);
                let pango_ctx = self
                    .da
                    .get_pango_context()
                    .expect("failed to get pango ctx");
                let linecount_layout =
                    self.create_layout_for_linecount(&pango_ctx, &main_state, i, padding);
                update_layout(cr, &linecount_layout);
                show_layout(cr, &linecount_layout);
            }
        }

        Inhibit(false)
    }

    pub fn handle_draw(&mut self, cr: &Context) -> Inhibit {
        // let foreground = self.main_state.borrow().theme.foreground;
        let theme = &self.main_state.borrow().theme;

        let da_width = self.da.get_allocated_width();
        let da_height = self.da.get_allocated_height();

        //debug!("Drawing");
        // cr.select_font_face("Mono", ::cairo::enums::FontSlant::Normal, ::cairo::enums::FontWeight::Normal);
        // let mut font_options = cr.get_font_options();
        // debug!("font options: {:?} {:?} {:?}", font_options, font_options.get_antialias(), font_options.get_hint_style());
        // font_options.set_hint_style(HintStyle::Full);

        // let (text_width, text_height) = self.get_text_size();
        let num_lines = self.line_cache.height();

        let vadj = self.vadj.clone();
        let hadj = self.hadj.clone();
        trace!("drawing.  vadj={}, {}", vadj.get_value(), vadj.get_upper());

        let first_line = (vadj.get_value() / self.font_height) as u64;
        let last_line = ((vadj.get_value() + f64::from(da_height)) / self.font_height) as u64 + 1;
        let last_line = min(last_line, num_lines);

        // debug!("line_cache {} {} {}", self.line_cache.n_invalid_before, self.line_cache.lines.len(), self.line_cache.n_invalid_after);
        // let missing = self.line_cache.get_missing(first_line, last_line);

        // Find missing lines
        let mut found_missing = false;
        for i in first_line..last_line {
            if self.line_cache.get_line(i).is_none() {
                debug!("missing line {}", i);
                found_missing = true;
            }
        }

        // We've already missed our chance to draw these lines, but we need to request them for the
        // next frame.  This needs to be improved to prevent flashing.
        if found_missing {
            debug!(
                "didn't have some lines, requesting, lines {}-{}",
                first_line, last_line
            );
            self.controller.borrow().core().request_lines(
                &self.view_id,
                first_line as u64,
                last_line as u64,
            );
        }

        let pango_ctx = self.da.get_pango_context().unwrap();
        pango_ctx.set_font_description(&self.font_desc);

        // Draw background
        set_source_color(cr, theme.background);
        cr.rectangle(0.0, 0.0, f64::from(da_width), f64::from(da_height));
        cr.fill();

        set_source_color(cr, theme.foreground);

        // Highlight cursor lines
        // for i in first_line..last_line {
        //     cr.set_source_rgba(0.8, 0.8, 0.8, 1.0);
        //     if let Some(line) = self.line_cache.get_line(i) {

        //         if !line.cursor().is_empty() {
        //             cr.set_source_rgba(0.23, 0.23, 0.23, 1.0);
        //             cr.rectangle(0f64,
        //                 font_extents.height*((i+1) as f64) - font_extents.ascent - vadj.get_value(),
        //                 da_width as f64,
        //                 font_extents.ascent + font_extents.descent);
        //             cr.fill();
        //         }
        //     }
        // }

        const CURSOR_WIDTH: f64 = 2.0;
        // Calculate ordinal or max line length
        let padding: usize = format!("{}", num_lines.saturating_sub(1)).len();

        let mut max_width = 0;

        let main_state = self.main_state.borrow();

        for i in first_line..last_line {
            // Keep track of the starting x position
            if let Some(line) = self.line_cache.get_line(i) {
                cr.move_to(
                    -hadj.get_value(),
                    self.font_height * (i as f64) - vadj.get_value(),
                );

                let pango_ctx = self
                    .da
                    .get_pango_context()
                    .expect("failed to get pango ctx");

                set_source_color(cr, theme.foreground);
                let layout = self.create_layout_for_line(&pango_ctx, &main_state, line);
                max_width = max(max_width, layout.get_extents().1.width);
                // debug!("width={}", layout.get_extents().1.width);
                update_layout(cr, &layout);
                show_layout(cr, &layout);

                let layout_line = layout.get_line(0);
                if layout_line.is_none() {
                    continue;
                }
                let layout_line = layout_line.unwrap();

                // Draw the cursor
                set_source_color(cr, theme.caret);

                for c in line.cursor() {
                    let x = layout_line.index_to_x(*c as i32, false) / pango::SCALE;
                    cr.rectangle(
                        (x as f64) - hadj.get_value(),
                        (((self.font_height) as u64) * i) as f64
                            - vadj.get_value(),
                        CURSOR_WIDTH,
                        self.font_height,
                    );
                    cr.fill();
                }
            }
        }

        // Now that we know actual length of the text, adjust the scrollbar properly.
        // But we need to make sure we don't make the upper value smaller than the current viewport
        let mut h_upper = f64::from(max_width / pango::SCALE);
        let cur_h_max = hadj.get_value() + hadj.get_page_size();
        if cur_h_max > h_upper {
            h_upper = cur_h_max;
        }

        if hadj.get_upper() != h_upper {
            hadj.set_upper(h_upper);
            // If I don't signal that the value changed, sometimes the overscroll "shadow" will stick
            // This seems to make sure to tell the viewport that something has changed so it can
            // reevaluate its need for a scroll shadow.
            hadj.value_changed();
        }

        Inhibit(false)
    }

    /// Creates a pango layout for a particular line number
    fn create_layout_for_linecount(
        &self,
        pango_ctx: &pango::Context,
        main_state: &MainState,
        n: u64,
        padding: usize,
    ) -> pango::Layout {
        let line_view = format!("{:>offset$} ", n, offset = padding);
        let layout = pango::Layout::new(pango_ctx);
        layout.set_font_description(Some(&self.font_desc));
        layout.set_text(line_view.as_str());
        layout
    }

    /// Creates a pango layout for a particular line in the linecache
    fn create_layout_for_line(
        &self,
        pango_ctx: &pango::Context,
        main_state: &MainState,
        line: &Line,
    ) -> pango::Layout {
        let line_view = if line.text().ends_with('\n') {
            &line.text()[0..line.text().len() - 1]
        } else {
            &line.text()
        };

        // let layout = create_layout(cr).unwrap();
        let layout = pango::Layout::new(pango_ctx);
        layout.set_font_description(Some(&self.font_desc));
        layout.set_text(line_view);

        let mut ix = 0;
        let attr_list = pango::AttrList::new();
        for style in &line.styles {
            let start_index = (ix + style.start) as u32;
            let end_index = (ix + style.start + style.len as i64) as u32;

            let foreground = main_state.styles.get(style.id).and_then(|s| s.fg_color);
            if let Some(foreground) = foreground {
                let mut attr = Attribute::new_foreground(
                    foreground.r_u16(),
                    foreground.g_u16(),
                    foreground.b_u16(),
                )
                .unwrap();
                attr.set_start_index(start_index);
                attr.set_end_index(end_index);
                attr_list.insert(attr);
            }

            let background = main_state.styles.get(style.id).and_then(|s| s.bg_color);
            if let Some(background) = background {
                let mut attr = Attribute::new_background(
                    background.r_u16(),
                    background.g_u16(),
                    background.b_u16(),
                )
                .unwrap();
                attr.set_start_index(start_index);
                attr.set_end_index(end_index);
                attr_list.insert(attr);
            }

            let weight = main_state.styles.get(style.id).and_then(|s| s.weight);
            if let Some(weight) = weight {
                let mut attr =
                    Attribute::new_weight(pango::Weight::__Unknown(weight as i32)).unwrap();
                attr.set_start_index(start_index);
                attr.set_end_index(end_index);
                attr_list.insert(attr);
            }

            let italic = main_state.styles.get(style.id).and_then(|s| s.italic);
            if let Some(italic) = italic {
                let mut attr = if italic {
                    Attribute::new_style(pango::Style::Italic).unwrap()
                } else {
                    Attribute::new_style(pango::Style::Normal).unwrap()
                };
                attr.set_start_index(start_index);
                attr.set_end_index(end_index);
                attr_list.insert(attr);
            }

            let underline = main_state.styles.get(style.id).and_then(|s| s.underline);
            if let Some(underline) = underline {
                let mut attr = if underline {
                    Attribute::new_underline(pango::Underline::Single).unwrap()
                } else {
                    Attribute::new_underline(pango::Underline::None).unwrap()
                };
                attr.set_start_index(start_index);
                attr.set_end_index(end_index);
                attr_list.insert(attr);
            }

            ix += style.start + style.len as i64;
        }

        layout.set_attributes(Some(&attr_list));
        layout
    }

    pub fn scroll_to(edit_view: &Rc<RefCell<EditView>>, line: u64, col: u64) {
        // We can't have edit_view borrowed when we call set_value on adjustments
        // because set_value can call the value_changed handlers.  So first, we
        // need to extract the information we're going to need.
        let (cur_top, cur_bottom, vadj, cur_left, cur_right, hadj) = {
            let ev = edit_view.borrow();
            let cur_top = ev.font_height * ((line + 1) as f64) - ev.font_ascent;
            let cur_bottom = cur_top + ev.font_ascent + ev.font_descent;
            let vadj = ev.vadj.clone();

            let cur_left = ev.font_width * (col as f64) - ev.font_ascent;
            let cur_right = cur_left + ev.font_width * 2.0;
            let hadj = ev.hadj.clone();

            (cur_top, cur_bottom, vadj, cur_left, cur_right, hadj)
        };

        if cur_top < vadj.get_value() {
            vadj.set_value(cur_top);
        } else if cur_bottom > vadj.get_value() + vadj.get_page_size()
            && vadj.get_page_size() != 0.0
        {
            vadj.set_value(cur_bottom - vadj.get_page_size());
        }

        if cur_left < hadj.get_value() {
            hadj.set_value(cur_left);
        } else if cur_right > hadj.get_value() + hadj.get_page_size() && hadj.get_page_size() != 0.0
        {
            let new_value = cur_right - hadj.get_page_size();
            if new_value + hadj.get_page_size() > hadj.get_upper() {
                hadj.set_upper(new_value + hadj.get_page_size());
            }
            hadj.set_value(new_value);
        }
    }

    pub fn handle_button_press(&self, eb: &EventButton) -> Inhibit {
        self.da.grab_focus();

        let (x, y) = eb.get_position();
        let (col, line) = {
            let main_state = self.main_state.borrow();
            self.da_px_to_cell(&main_state, x, y)
        };

        match eb.get_button() {
            1 => {
                if eb.get_state().contains(ModifierType::SHIFT_MASK) {
                    self.controller
                        .borrow()
                        .core()
                        .gesture_range_select(&self.view_id, line, col);
                } else if eb.get_state().contains(ModifierType::CONTROL_MASK) {
                    self.controller
                        .borrow()
                        .core()
                        .gesture_toggle_sel(&self.view_id, line, col);
                } else if eb.get_event_type() == EventType::DoubleButtonPress {
                    self.controller
                        .borrow()
                        .core()
                        .gesture_word_select(&self.view_id, line, col);
                } else if eb.get_event_type() == EventType::TripleButtonPress {
                    self.controller
                        .borrow()
                        .core()
                        .gesture_line_select(&self.view_id, line, col);
                } else {
                    self.controller
                        .borrow()
                        .core()
                        .gesture_point_select(&self.view_id, line, col);
                }
            }
            2 => {
                self.do_paste_primary(&self.view_id, line, col);
            }
            _ => {}
        }
        Inhibit(false)
    }

    pub fn handle_drag(&mut self, em: &EventMotion) -> Inhibit {
        let (x, y) = em.get_position();
        let (col, line) = {
            let main_state = self.main_state.borrow();
            self.da_px_to_cell(&main_state, x, y)
        };
        self.controller.borrow().core().drag(
            &self.view_id,
            line,
            col,
            convert_gtk_modifier(em.get_state()),
        );
        Inhibit(false)
    }

    pub fn handle_scroll(&mut self, es: &EventScroll) -> Inhibit {
        // self.da.grab_focus();
        // // let amt = self.font_height * 3.0;

        // if let ScrollDirection::Smooth = es.get_direction() {
        //     error!("Smooth scroll!");
        // }

        // debug!("handle scroll {:?}", es);
        // let vadj = self.vadj.clone();
        // let hadj = self.hadj.clone();
        // match es.get_direction() {
        //     ScrollDirection::Up => vadj.set_value(vadj.get_value() - amt),
        //     ScrollDirection::Down => vadj.set_value(vadj.get_value() + amt),
        //     ScrollDirection::Left => hadj.set_value(hadj.get_value() - amt),
        //     ScrollDirection::Right => hadj.set_value(hadj.get_value() + amt),
        //     ScrollDirection::Smooth => debug!("scroll Smooth"),
        //     _ => {},
        // }

        self.update_visible_scroll_region();

        Inhibit(false)
    }

    fn handle_key_press_event(&mut self, ek: &EventKey) -> Inhibit {
        debug!(
            "key press keyval={:?}, state={:?}, length={:?} group={:?} uc={:?}",
            ek.get_keyval(),
            ek.get_state(),
            ek.get_length(),
            ek.get_group(),
            ::gdk::keyval_to_unicode(ek.get_keyval())
        );
        let view_id = &self.view_id;
        let ch = ::gdk::keyval_to_unicode(ek.get_keyval());

        let alt = ek.get_state().contains(ModifierType::MOD1_MASK);
        let ctrl = ek.get_state().contains(ModifierType::CONTROL_MASK);
        let meta = ek.get_state().contains(ModifierType::META_MASK);
        let shift = ek.get_state().contains(ModifierType::SHIFT_MASK);
        let norm = !alt && !ctrl && !meta;

        match ek.get_keyval() {
            key::Delete if norm => self.controller.borrow().core().delete_forward(view_id),
            key::BackSpace if norm => self.controller.borrow().core().delete_backward(view_id),
            key::Return | key::KP_Enter => {
                self.controller.borrow().core().insert_newline(&view_id);
            }
            key::Tab if norm && !shift => self.controller.borrow().core().insert_tab(view_id),
            key::Up if norm && !shift => self.controller.borrow().core().move_up(view_id),
            key::Down if norm && !shift => self.controller.borrow().core().move_down(view_id),
            key::Left if norm && !shift => self.controller.borrow().core().move_left(view_id),
            key::Right if norm && !shift => self.controller.borrow().core().move_right(view_id),
            key::Up if norm && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_up_and_modify_selection(view_id);
            }
            key::Down if norm && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_down_and_modify_selection(view_id);
            }
            key::Left if norm && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_left_and_modify_selection(view_id);
            }
            key::Right if norm && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_right_and_modify_selection(view_id);
            }
            key::Left if ctrl && !shift => {
                self.controller.borrow().core().move_word_left(view_id);
            }
            key::Right if ctrl && !shift => {
                self.controller.borrow().core().move_word_right(view_id);
            }
            key::Left if ctrl && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_word_left_and_modify_selection(view_id);
            }
            key::Right if ctrl && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_word_right_and_modify_selection(view_id);
            }
            key::Home if norm && !shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_to_left_end_of_line(view_id);
            }
            key::End if norm && !shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_to_right_end_of_line(view_id);
            }
            key::Home if norm && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_to_left_end_of_line_and_modify_selection(view_id);
            }
            key::End if norm && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_to_right_end_of_line_and_modify_selection(view_id);
            }
            key::Home if ctrl && !shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_to_beginning_of_document(view_id);
            }
            key::End if ctrl && !shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_to_end_of_document(view_id);
            }
            key::Home if ctrl && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_to_beginning_of_document_and_modify_selection(view_id);
            }
            key::End if ctrl && shift => {
                self.controller
                    .borrow()
                    .core()
                    .move_to_end_of_document_and_modify_selection(view_id);
            }
            key::Page_Up if norm && !shift => {
                self.controller.borrow().core().page_up(view_id);
            }
            key::Page_Down if norm && !shift => {
                self.controller.borrow().core().page_down(view_id);
            }
            key::Page_Up if norm && shift => {
                self.controller
                    .borrow()
                    .core()
                    .page_up_and_modify_selection(view_id);
            }
            key::Page_Down if norm && shift => {
                self.controller
                    .borrow()
                    .core()
                    .page_down_and_modify_selection(view_id);
            }
            _ => {
                if let Some(ch) = ch {
                    match ch {
                        'a' if ctrl => {
                            self.controller.borrow().core().select_all(view_id);
                        }
                        'c' if ctrl => {
                            self.do_copy(view_id);
                        }
                        'f' if ctrl => {
                            self.start_search();
                        }
                        'v' if ctrl => {
                            self.do_paste(view_id);
                        }
                        't' if ctrl => {
                            // TODO new tab
                        }
                        'x' if ctrl => {
                            self.do_cut(view_id);
                        }
                        'z' if ctrl => {
                            self.controller.borrow().core().undo(view_id);
                        }
                        'Z' if ctrl && shift => {
                            self.controller.borrow().core().redo(view_id);
                        }
                        c if (norm) && c >= '\u{0020}' => {
                            debug!("inserting key");
                            self.controller
                                .borrow()
                                .core()
                                .insert(view_id, &c.to_string());
                        }
                        _ => {
                            debug!("unhandled key: {:?}", ch);
                        }
                    }
                }
            }
        };
        Inhibit(true)
    }

    fn do_cut(&self, view_id: &str) {
        if let Some(text) = self.controller.borrow().core().cut(view_id) {
            Clipboard::get(&SELECTION_CLIPBOARD).set_text(&text);
        }
    }

    fn do_copy(&self, view_id: &str) {
        if let Some(text) = self.controller.borrow().core().copy(view_id) {
            Clipboard::get(&SELECTION_CLIPBOARD).set_text(&text);
        }
    }

    fn do_paste(&self, view_id: &str) {
        let view_id2 = view_id.to_string().clone();
        let core = self.controller.borrow().core().clone();
        Clipboard::get(&SELECTION_CLIPBOARD).request_text(move |_, text| {
            if let Some(text) = text {
                core.paste(&view_id2, &text);
            }
        });
    }

    fn do_paste_primary(&self, view_id: &str, line: u64, col: u64) {
        let view_id2 = view_id.to_string().clone();
        let core = self.controller.borrow().core().clone();
        Clipboard::get(&SELECTION_PRIMARY).request_text(move |_, text| {
            if let Some(text) = text {
                core.gesture_point_select(&view_id2, line, col);
                core.insert(&view_id2, text);
            }
        });
    }

    pub fn start_search(&self) {
        self.search_bar.set_search_mode(true);
        self.replace_expander.set_expanded(false);
        self.replace_revealer.set_reveal_child(false);
        self.search_entry.grab_focus();
        let needle = self
            .search_entry
            .get_text()
            .map(|gs| gs.as_str().to_owned())
            .unwrap_or_default();
        self.controller
            .borrow()
            .core()
            .find(&self.view_id, needle, false, Some(false));
    }

    pub fn stop_search(&self) {
        self.search_bar.set_search_mode(false);
        self.da.grab_focus();
    }

    pub fn find_status(&self, queries: &Value) {
        if let Some(queries) = queries.as_array() {
            for query in queries {
                if let Some(query_obj) = query.as_object() {
                    if let Some(matches) = query_obj["matches"].as_u64() {
                        self.find_status_label
                            .set_text(&format!("{} Results", matches));
                    }
                }
                debug!("query {}", query);
            }
        }
    }

    pub fn find_next(&self) {
        self.controller
            .borrow()
            .core()
            .find_next(&self.view_id, Some(true), Some(true));
    }

    pub fn find_prev(&self) {
        self.controller
            .borrow()
            .core()
            .find_previous(&self.view_id, Some(true));
    }

    pub fn search_changed(&self, s: Option<String>) {
        let needle = s.unwrap_or_default();
        self.controller
            .borrow()
            .core()
            .find(&self.view_id, needle, false, Some(false));
    }

    pub fn replace(&self) {
        let replace_chars = self
            .replace_entry
            .get_text()
            .map(|gs| gs.as_str().to_owned())
            .unwrap_or_default();
        self.controller
            .borrow()
            .core()
            .replace(&self.view_id, &replace_chars, false);
        self.controller.borrow().core().replace_next(&self.view_id);
    }

    pub fn replace_all(&self) {
        let replace_chars = self
            .replace_entry
            .get_text()
            .map(|gs| gs.as_str().to_owned())
            .unwrap_or_default();
        self.controller
            .borrow()
            .core()
            .replace(&self.view_id, &replace_chars, false);
        self.controller.borrow().core().replace_all(&self.view_id);
    }
}
