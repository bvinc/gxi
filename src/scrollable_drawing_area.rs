/**
 * This file is a monstrosity.  It was generated from the gnome-class project.
 * Due to various reasons, I could not use the gnome-class macro, or the
 * glib object wrapper macro.  One day, I hope to be able to use gnome-class.
 *
 * The entire point of this file is to create a custom widget that extends
 * DrawingArea and implements Scrollable.  This was the only way I could find
 * to get nice GTK auto-hiding scrollbars.
 *
 * This probably needs cleanup.
 */
use std;

use glib::translate::*;
use glib::StaticType;
use gobject_sys;
use gtk::{Adjustment, DrawingArea, Scrollable, ScrollablePolicy};
use gtk::{Buildable, Widget};
use gtk_sys::{GtkAdjustment, GtkDrawingArea, GtkWidget};
use libc::c_int;

use std::cell::Cell;
use std::clone::Clone;
use std::fmt::Debug;
use std::hash::Hash;

/********************************************************************************/
extern crate glib;
extern crate glib_sys as glib_ffi;
extern crate gobject_sys as gobject_ffi;
use glib::{glib_object_wrapper, glib_wrapper, Cast, IsA};
use std::mem;
use std::ptr;

glib_wrapper! {
    pub struct ScrollableDrawingArea(Object<imp::ScrollableDrawingAreaFfi, imp::ScrollableDrawingAreaClass, ScrollableDrawingAreaClass>)
        @extends DrawingArea, Widget, // parent classes
        @implements Buildable, Scrollable;  // interfaces

    match fn {
        get_type => || imp::scrollable_drawing_area_get_type(),
    }
}

pub mod imp {
    use super::super::*;
    use super::glib;
    use super::glib_ffi;
    use super::gobject_ffi;
    use super::ScrollableDrawingArea;
    use glib::translate::*;
    use glib::Cast;
    use gtk::{Adjustment, DrawingArea, Scrollable, ScrollablePolicy};
    use gtk_sys::{GtkAdjustment, GtkDrawingArea, GtkDrawingAreaClass, GtkWidget};
    use libc::c_int;
    use std::cell::Cell;
    use std::ffi::CString;
    use std::mem;
    use std::ptr;

    #[repr(C)]
    #[derive(Debug)]
    pub struct ScrollableDrawingAreaFfi {
        pub parent: GtkDrawingArea,
    }
    #[repr(C)]
    #[derive(Debug)]
    pub struct ScrollableDrawingAreaClass {
        pub parent_class: GtkDrawingAreaClass,
        pub value_changed:
            Option<unsafe extern "C" fn(this: *mut ScrollableDrawingAreaFfi) -> (())>,
        pub size_allocate: Option<
            unsafe extern "C" fn(this: *mut ScrollableDrawingAreaFfi, i: u32, j: u32) -> u32,
        >,
    }
    #[allow(non_camel_case_types)]
    #[repr(u32)]
    #[derive(Debug)]
    enum Properties {
        hadjustment = 1u32,
        hscroll_policy = 2u32,
        vadjustment = 3u32,
        vscroll_policy = 4u32,
    }
    #[derive(Debug)]
    struct ScrollableDrawingAreaClassPrivate {
        parent_class: *const GtkDrawingAreaClass,
        properties: *const Vec<*const gobject_ffi::GParamSpec>,
        value_changed_signal_id: u32,
    }
    static mut PRIV: ScrollableDrawingAreaClassPrivate = ScrollableDrawingAreaClassPrivate {
        parent_class: 0 as *const _,
        properties: 0 as *const _,
        value_changed_signal_id: 0,
    };
    #[derive(Debug)]
    struct ScrollableDrawingAreaPriv {
        hadjustment: RefCell<Adjustment>,
        hscroll_policy: Cell<c_int>,
        vadjustment: RefCell<Adjustment>,
        vscroll_policy: Cell<c_int>,
    }
    #[allow(unused_qualifications)]
    impl Default for ScrollableDrawingAreaPriv {
        #[inline]
        fn default() -> ScrollableDrawingAreaPriv {
            ScrollableDrawingAreaPriv {
                hadjustment: RefCell::new(Adjustment::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
                hscroll_policy: Default::default(),
                vadjustment: RefCell::new(Adjustment::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
                vscroll_policy: Default::default(),
            }
        }
    }
    impl super::ScrollableDrawingArea {
        #[allow(dead_code)]
        fn get_priv(&self) -> &ScrollableDrawingAreaPriv {
            unsafe {
                let _private = gobject_ffi::g_type_instance_get_private(
                    <Self as ToGlibPtr<*mut ScrollableDrawingAreaFfi>>::to_glib_none(self).0
                        as *mut gobject_ffi::GTypeInstance,
                    scrollable_drawing_area_get_type(),
                ) as *const Option<ScrollableDrawingAreaPriv>;
                (&*_private).as_ref().unwrap()
            }
        }
        #[allow(unused_variables)]
        fn value_changed_impl(&self) -> (()) {
            {
                panic!("Called default signal handler with no implementation")
            };
        }
        fn size_allocate_impl(&self, i: u32, j: u32) -> u32 {
            1 + i + j
        }
        #[allow(unused)]
        fn emit_value_changed(&self) -> (()) {
            let params: &[glib::Value] = &[(self as &glib::ToValue).to_value()];
            unsafe {
                let mut ret = glib::Value::uninitialized();
                gobject_sys::g_signal_emitv(
                    mut_override(params.as_ptr()) as *mut gobject_sys::GValue,
                    PRIV.value_changed_signal_id,
                    0,
                    ret.to_glib_none_mut().0,
                );
                ()
            }
        }
        #[allow(dead_code, unused_variables)]
        fn set_property_impl(&self, property_id: u32, value: *mut gobject_ffi::GValue) {
            match property_id {
                1u32 => {
                    let v: glib::Value = unsafe { FromGlibPtrNone::from_glib_none(value) };
                    if let Ok(Some(value)) = v.get::<Adjustment>() {
                        let mut private = self.get_priv();
                        *private.hadjustment.borrow_mut() = value.clone();
                    }
                }
                2u32 => {
                    let v: glib::Value = unsafe { FromGlibPtrNone::from_glib_none(value) };
                    if let Ok(Some(value)) = v.get::<c_int>() {
                        let mut private = self.get_priv();
                        private.hscroll_policy.set(value);
                    }
                }
                3u32 => {
                    let v: glib::Value = unsafe { FromGlibPtrNone::from_glib_none(value) };
                    if let Ok(Some(value)) = v.get::<Adjustment>() {
                        let mut private = self.get_priv();
                        *private.vadjustment.borrow_mut() = value.clone();
                    }
                }
                4u32 => {
                    let v: glib::Value = unsafe { FromGlibPtrNone::from_glib_none(value) };
                    if let Ok(Some(value)) = v.get::<c_int>() {
                        let mut private = self.get_priv();
                        private.vscroll_policy.set(value);
                    }
                }
                _ => {}
            }
        }
        #[allow(dead_code, unused_variables)]
        fn get_property_impl(&self, property_id: u32, value: *mut gobject_ffi::GValue) {
            match property_id {
                1u32 => {
                    let ret: *mut ::gobject_sys::GObject = (|| {
                        let private = self.get_priv();
                        let ptr: *mut GtkAdjustment = private.hadjustment.borrow().to_glib_none().0;
                        return ptr as *mut ::gobject_sys::GObject;
                    })();
                    unsafe {
                        gobject_ffi::g_value_set_object(value, ret);
                    }
                }
                2u32 => {
                    let ret: c_int = (|| {
                        let private = self.get_priv();
                        return private.hscroll_policy.get();
                    })();
                    unsafe {
                        gobject_ffi::g_value_set_enum(value, ret);
                    }
                }
                3u32 => {
                    let ret: *mut ::gobject_sys::GObject = (|| {
                        let private = self.get_priv();
                        let ptr: *mut GtkAdjustment = private.vadjustment.borrow().to_glib_none().0;
                        return ptr as *mut ::gobject_sys::GObject;
                    })();
                    unsafe {
                        gobject_ffi::g_value_set_object(value, ret);
                    }
                }
                4u32 => {
                    let ret: c_int = (|| {
                        let private = self.get_priv();
                        return private.vscroll_policy.get();
                    })();
                    unsafe {
                        gobject_ffi::g_value_set_enum(value, ret);
                    }
                }
                _ => {}
            }
        }
    }
    impl ScrollableDrawingAreaFfi {
        #[allow(dead_code)]
        fn get_vtable(&self) -> &ScrollableDrawingAreaClass {
            unsafe {
                let klass = (*(self as *const _ as *const gobject_ffi::GTypeInstance)).g_class;
                &*(klass as *const ScrollableDrawingAreaClass)
            }
        }
        unsafe extern "C" fn init(
            obj: *mut gobject_ffi::GTypeInstance,
            _klass: glib_ffi::gpointer,
        ) {
            #[allow(unused_variables)]
            let obj = obj;
            #[allow(deprecated)]
            let _guard = glib::CallbackGuard::new();
            let _private =
                gobject_ffi::g_type_instance_get_private(obj, scrollable_drawing_area_get_type())
                    as *mut Option<ScrollableDrawingAreaPriv>;
            ptr::write(
                _private,
                Some(<ScrollableDrawingAreaPriv as Default>::default()),
            );
        }
        unsafe extern "C" fn finalize(obj: *mut gobject_ffi::GObject) {
            #[allow(deprecated)]
            let _guard = glib::CallbackGuard::new();
            let _private = gobject_ffi::g_type_instance_get_private(
                obj as *mut gobject_ffi::GTypeInstance,
                scrollable_drawing_area_get_type(),
            ) as *mut Option<ScrollableDrawingAreaPriv>;
            let _ = (*_private).take();
            (*(PRIV.parent_class as *mut gobject_ffi::GObjectClass))
                .finalize
                .map(|f| f(obj));
        }
        unsafe extern "C" fn set_property(
            obj: *mut gobject_ffi::GObject,
            property_id: u32,
            value: *mut gobject_ffi::GValue,
            _pspec: *mut gobject_ffi::GParamSpec,
        ) {
            #[allow(deprecated)]
            let _guard = glib::CallbackGuard::new();
            let this: &ScrollableDrawingArea =
                &ScrollableDrawingArea::from_glib_borrow(obj as *mut ScrollableDrawingAreaFfi);
            this.set_property_impl(property_id, value);
        }
        unsafe extern "C" fn get_property(
            obj: *mut gobject_ffi::GObject,
            property_id: u32,
            value: *mut gobject_ffi::GValue,
            _pspec: *mut gobject_ffi::GParamSpec,
        ) {
            #[allow(deprecated)]
            let _guard = glib::CallbackGuard::new();
            let this: &ScrollableDrawingArea =
                &ScrollableDrawingArea::from_glib_borrow(obj as *mut ScrollableDrawingAreaFfi);
            this.get_property_impl(property_id, value);
        }
        unsafe extern "C" fn value_changed_slot_trampoline(
            this: *mut ScrollableDrawingAreaFfi,
        ) -> (()) {
            #[allow(deprecated)]
            let _guard = glib::CallbackGuard::new();
            let this = this as *mut ScrollableDrawingAreaFfi;
            let instance: &super::ScrollableDrawingArea = &from_glib_borrow(this);
            instance.value_changed_impl()
        }
        unsafe extern "C" fn size_allocate_slot_trampoline(
            this: *mut ScrollableDrawingAreaFfi,
            i: u32,
            j: u32,
        ) -> u32 {
            #[allow(deprecated)]
            let _guard = glib::CallbackGuard::new();
            let this = this as *mut ScrollableDrawingAreaFfi;
            let instance: &super::ScrollableDrawingArea = &from_glib_borrow(this);
            instance.size_allocate_impl(i, j)
        }
    }
    impl ScrollableDrawingAreaClass {
        unsafe extern "C" fn init(vtable: glib_ffi::gpointer, _klass_data: glib_ffi::gpointer) {
            #[allow(deprecated)]
            let _guard = glib::CallbackGuard::new();
            gobject_ffi::g_type_class_add_private(
                vtable,
                mem::size_of::<Option<ScrollableDrawingAreaPriv>>(),
            );
            {
                let gobject_class = &mut *(vtable as *mut gobject_ffi::GObjectClass);
                gobject_class.finalize = Some(ScrollableDrawingAreaFfi::finalize);
                gobject_class.set_property = Some(ScrollableDrawingAreaFfi::set_property);
                gobject_class.get_property = Some(ScrollableDrawingAreaFfi::get_property);
                let mut properties = Vec::new();
                properties.push(ptr::null());
                properties.push(gobject_ffi::g_param_spec_object(
                    CString::new("hadjustment").unwrap().as_ptr(),
                    CString::new("hadjustment").unwrap().as_ptr(),
                    CString::new("hadjustment").unwrap().as_ptr(),
                    ::gtk_sys::gtk_adjustment_get_type(),
                    gobject_ffi::G_PARAM_READWRITE | gobject_ffi::G_PARAM_CONSTRUCT,
                ));
                properties.push(gobject_ffi::g_param_spec_enum(
                    CString::new("hscroll-policy").unwrap().as_ptr(),
                    CString::new("hscroll-policy").unwrap().as_ptr(),
                    CString::new("hscroll-policy").unwrap().as_ptr(),
                    ::gtk_sys::gtk_scrollable_policy_get_type(),
                    ScrollablePolicy::Minimum.to_glib(),
                    gobject_ffi::G_PARAM_READWRITE,
                ));
                properties.push(gobject_ffi::g_param_spec_object(
                    CString::new("vadjustment").unwrap().as_ptr(),
                    CString::new("vadjustment").unwrap().as_ptr(),
                    CString::new("vadjustment").unwrap().as_ptr(),
                    ::gtk_sys::gtk_adjustment_get_type(),
                    gobject_ffi::G_PARAM_READWRITE | gobject_ffi::G_PARAM_CONSTRUCT,
                ));
                properties.push(gobject_ffi::g_param_spec_enum(
                    CString::new("vscroll-policy").unwrap().as_ptr(),
                    CString::new("vscroll-policy").unwrap().as_ptr(),
                    CString::new("vscroll-policy").unwrap().as_ptr(),
                    ::gtk_sys::gtk_scrollable_policy_get_type(),
                    ScrollablePolicy::Minimum.to_glib(),
                    gobject_ffi::G_PARAM_READWRITE,
                ));
                if properties.len() > 1 {
                    gobject_ffi::g_object_class_install_properties(
                        gobject_class,
                        properties.len() as u32,
                        properties.as_mut_ptr() as *mut *mut _,
                    );
                }
                PRIV.properties = Box::into_raw(Box::new(properties));
            }
            {
                #[allow(unused_variables)]
                let vtable = &mut *(vtable as *mut ScrollableDrawingAreaClass);
                vtable.value_changed =
                    Some(ScrollableDrawingAreaFfi::value_changed_slot_trampoline);
                vtable.size_allocate =
                    Some(ScrollableDrawingAreaFfi::size_allocate_slot_trampoline);
            }
            {
                let param_gtypes = [];
                PRIV.value_changed_signal_id = gobject_sys::g_signal_newv(
                    b"value-changed\x00" as *const u8 as *const i8,
                    scrollable_drawing_area_get_type(),
                    gobject_sys::G_SIGNAL_RUN_LAST,
                    ptr::null_mut(),
                    None,
                    ptr::null_mut(),
                    None,
                    gobject_sys::G_TYPE_NONE,
                    0u32,
                    mut_override(param_gtypes.as_ptr()),
                );
            }
            PRIV.parent_class =
                gobject_ffi::g_type_class_peek_parent(vtable) as *const GtkDrawingAreaClass;
        }
    }
    pub unsafe extern "C" fn scrollable_drawing_area_new() -> *mut ScrollableDrawingAreaFfi {
        #[allow(deprecated)]
        let _guard = glib::CallbackGuard::new();
        let this =
            gobject_ffi::g_object_newv(scrollable_drawing_area_get_type(), 0, ptr::null_mut());
        this as *mut ScrollableDrawingAreaFfi
    }
    pub unsafe extern "C" fn scrollable_drawing_area_size_allocate(
        this: *mut ScrollableDrawingAreaFfi,
        i: u32,
        j: u32,
    ) -> u32 {
        #[allow(deprecated)]
        let _guard = glib::CallbackGuard::new();
        let vtable = (*this).get_vtable();
        (vtable.size_allocate.as_ref().unwrap())(this, i, j)
    }
    pub unsafe extern "C" fn scrollable_drawing_area_get_type() -> glib_ffi::GType {
        #[allow(deprecated)]
        let _guard = glib::CallbackGuard::new();
        use std::sync::{Once, ONCE_INIT};
        use std::u16;
        static mut TYPE: glib_ffi::GType = gobject_ffi::G_TYPE_INVALID;
        static ONCE: Once = ONCE_INIT;
        ONCE.call_once(|| {
            let class_size = mem::size_of::<ScrollableDrawingAreaClass>();
            if !(class_size <= u16::MAX as usize) {
                {
                    panic!("assertion failed: class_size <= u16::MAX as usize")
                }
            };
            let instance_size = mem::size_of::<ScrollableDrawingAreaFfi>();
            if !(instance_size <= u16::MAX as usize) {
                {
                    panic!("assertion failed: instance_size <= u16::MAX as usize")
                }
            };
            TYPE = gobject_ffi::g_type_register_static_simple(
                <DrawingArea as glib::StaticType>::static_type().to_glib(),
                b"ScrollableDrawingArea\x00" as *const u8 as *const i8,
                class_size as u32,
                Some(ScrollableDrawingAreaClass::init),
                instance_size as u32,
                Some(ScrollableDrawingAreaFfi::init),
                0,
            );

            let interface_info = gobject_ffi::GInterfaceInfo {
                interface_init: None,            //TODO
                interface_finalize: None,        //TODO
                interface_data: ptr::null_mut(), //TODO
            };
            gobject_ffi::g_type_add_interface_static(
                TYPE,
                <Scrollable as glib::StaticType>::static_type().to_glib(),
                &interface_info,
            );
        });
        TYPE
    }
}
impl ScrollableDrawingArea {
    pub fn new() -> ScrollableDrawingArea {
        unsafe { from_glib_full(imp::scrollable_drawing_area_new()) }
    }
}
pub trait ScrollableDrawingAreaExt {
    fn connect_value_changed<F: Fn(&Self) -> (()) + 'static>(&self, f: F) -> glib::SignalHandlerId;
    fn size_allocate(&self, i: u32, j: u32) -> u32;
    fn get_property_hadjustment(&self) -> Adjustment;
    fn get_property_hscroll_policy(&self) -> u32;
    fn get_property_vadjustment(&self) -> Adjustment;
    fn get_property_vscroll_policy(&self) -> u32;
    fn set_property_hadjustment(&self, v: Adjustment);
    fn set_property_hscroll_policy(&self, v: u32);
    fn set_property_vadjustment(&self, v: Adjustment);
    fn set_property_vscroll_policy(&self, v: u32);
}
impl<O: IsA<ScrollableDrawingArea> + IsA<glib::object::Object> + glib::object::ObjectExt>
    ScrollableDrawingAreaExt for O
{
    fn connect_value_changed<F: Fn(&Self) -> (()) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        unsafe {
            let f: Box<Box<Fn(&Self) -> (()) + 'static>> = Box::new(Box::new(f));
            glib::signal::connect_raw(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "value_changed".to_glib_none().0,
                mem::transmute(value_changed_signal_handler_trampoline::<Self> as usize),
                Box::into_raw(f) as *mut _,
            )
        }
    }
    fn size_allocate(&self, i: u32, j: u32) -> u32 {
        unsafe {
            imp::scrollable_drawing_area_size_allocate(
                self.to_glib_none().0 as *mut imp::ScrollableDrawingAreaFfi,
                i,
                j,
            )
        }
    }
    fn get_property_hadjustment(&self) -> Adjustment {
        let mut value = glib::Value::from(&Adjustment::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        unsafe {
            gobject_ffi::g_object_get_property(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "hadjustment".to_glib_none().0,
                value.to_glib_none_mut().0,
            );
        }
        value.get::<Adjustment>().unwrap().unwrap()
    }
    fn get_property_hscroll_policy(&self) -> u32 {
        let mut value = glib::Value::from(&u32::default());
        unsafe {
            gobject_ffi::g_object_get_property(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "hscroll-policy".to_glib_none().0,
                value.to_glib_none_mut().0,
            );
        }
        value.get::<u32>().unwrap().unwrap()
    }
    fn get_property_vadjustment(&self) -> Adjustment {
        let mut value = glib::Value::from(&Adjustment::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        unsafe {
            gobject_ffi::g_object_get_property(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "vadjustment".to_glib_none().0,
                value.to_glib_none_mut().0,
            );
        }
        value.get::<Adjustment>().unwrap().unwrap()
    }
    fn get_property_vscroll_policy(&self) -> u32 {
        let mut value = glib::Value::from(&u32::default());
        unsafe {
            gobject_ffi::g_object_get_property(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "vscroll-policy".to_glib_none().0,
                value.to_glib_none_mut().0,
            );
        }
        value.get::<u32>().unwrap().unwrap()
    }
    fn set_property_hadjustment(&self, v: Adjustment) {
        unsafe {
            gobject_ffi::g_object_set_property(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "hadjustment".to_glib_none().0,
                glib::Value::from(&v).to_glib_none().0,
            );
        }
    }
    fn set_property_hscroll_policy(&self, v: u32) {
        unsafe {
            gobject_ffi::g_object_set_property(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "hscroll-policy".to_glib_none().0,
                glib::Value::from(&v).to_glib_none().0,
            );
        }
    }
    fn set_property_vadjustment(&self, v: Adjustment) {
        unsafe {
            gobject_ffi::g_object_set_property(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "vadjustment".to_glib_none().0,
                glib::Value::from(&v).to_glib_none().0,
            );
        }
    }
    fn set_property_vscroll_policy(&self, v: u32) {
        unsafe {
            gobject_ffi::g_object_set_property(
                self.to_glib_none().0 as *mut gobject_sys::GObject,
                "vscroll-policy".to_glib_none().0,
                glib::Value::from(&v).to_glib_none().0,
            );
        }
    }
}
unsafe extern "C" fn value_changed_signal_handler_trampoline<P>(
    this: *mut imp::ScrollableDrawingAreaFfi,
    f: glib_ffi::gpointer,
) -> (())
where
    P: IsA<ScrollableDrawingArea>,
{
    #[allow(deprecated)]
    let _guard = glib::CallbackGuard::new();
    let f: &&(Fn(&P) -> (()) + 'static) = mem::transmute(f);
    f(&ScrollableDrawingArea::from_glib_borrow(this).unsafe_cast())
}
