use crate::app::AppState;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{EventControllerScroll, EventControllerScrollFlags, GestureDrag, PropagationPhase};
use std::cell::RefCell;
use std::rc::Rc;

mod imp {
    use super::*;

    // Wrap Rc<RefCell<...>> so the GObject subclass system is satisfied.
    // Safety: AppState is only ever accessed from the GTK main thread.
    struct MainThreadOnly<T>(T);
    unsafe impl<T> Send for MainThreadOnly<T> {}
    unsafe impl<T> Sync for MainThreadOnly<T> {}

    pub struct ImageCanvas {
        state: MainThreadOnly<RefCell<Option<Rc<RefCell<AppState>>>>>,
    }

    impl Default for ImageCanvas {
        fn default() -> Self {
            Self {
                state: MainThreadOnly(RefCell::new(None)),
            }
        }
    }

    impl ImageCanvas {
        pub fn get_state(&self) -> std::cell::Ref<'_, Option<Rc<RefCell<AppState>>>> {
            self.state.0.borrow()
        }

        pub fn set_state(&self, state: Rc<RefCell<AppState>>) {
            *self.state.0.borrow_mut() = Some(state);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageCanvas {
        const NAME: &'static str = "ImageCanvas";
        type Type = super::ImageCanvas;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for ImageCanvas {}

    impl WidgetImpl for ImageCanvas {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let widget = self.obj();
            let w = widget.width() as f32;
            let h = widget.height() as f32;

            snapshot.append_color(
                &gtk4::gdk::RGBA::new(0.15, 0.15, 0.15, 1.0),
                &gtk4::graphene::Rect::new(0.0, 0.0, w, h),
            );

            let state_guard = self.get_state();
            if let Some(state_rc) = state_guard.as_ref() {
                let state = state_rc.borrow();
                if let Some(texture) = state.current_texture() {
                    let scale = state.viewer.effective_scale();
                    let (ox, oy) = state.viewer.draw_offset();
                    let img_w = state.viewer.image_size.0 * scale;
                    let img_h = state.viewer.image_size.1 * scale;
                    let rect = gtk4::graphene::Rect::new(
                        ox as f32,
                        oy as f32,
                        img_w as f32,
                        img_h as f32,
                    );
                    let filter = if scale >= 2.0 {
                        gtk4::gsk::ScalingFilter::Nearest
                    } else {
                        gtk4::gsk::ScalingFilter::Linear
                    };
                    snapshot.append_scaled_texture(texture, filter, &rect);
                }
            }
        }

        fn size_allocate(&self, w: i32, h: i32, baseline: i32) {
            self.parent_size_allocate(w, h, baseline);
            let state_guard = self.get_state();
            if let Some(state_rc) = state_guard.as_ref() {
                let mut st = state_rc.borrow_mut();
                st.viewer.canvas_size = (w as f64, h as f64);
                st.viewer.clamp_pan();
            }
        }

        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            (0, 0, -1, -1)
        }
    }
}

glib::wrapper! {
    pub struct ImageCanvas(ObjectSubclass<imp::ImageCanvas>)
        @extends gtk4::Widget;
}

impl ImageCanvas {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().set_state(state);
        obj.set_hexpand(true);
        obj.set_vexpand(true);
        obj.set_can_focus(true);
        obj.set_focusable(true);
        obj.setup_controllers();
        obj
    }

    fn setup_controllers(&self) {
        // --- Mouse wheel: zoom toward cursor ---
        let scroll = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        scroll.set_propagation_phase(PropagationPhase::Target);

        let weak = self.downgrade();
        scroll.connect_scroll(move |ctrl, _dx, dy| {
            let Some(this) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            let (w, h) = ctrl
                .widget()
                .map(|w| (w.width() as f64, w.height() as f64))
                .unwrap_or((800.0, 600.0));
            let (px, py) = ctrl
                .current_event()
                .and_then(|e| e.position())
                .unwrap_or((w / 2.0, h / 2.0));

            {
                let state_guard = this.imp().get_state();
                if let Some(state_rc) = state_guard.as_ref() {
                    state_rc.borrow_mut().viewer.zoom_toward(-dy, px, py);
                }
            }
            this.queue_draw();
            glib::Propagation::Stop
        });
        self.add_controller(scroll);

        // --- Drag: pan ---
        let drag = GestureDrag::new();
        drag.set_propagation_phase(PropagationPhase::Target);

        let pan_at_start: Rc<RefCell<(f64, f64)>> = Rc::new(RefCell::new((0.0, 0.0)));
        let pan_at_start2 = pan_at_start.clone();

        let weak = self.downgrade();
        drag.connect_drag_begin(move |_, _x, _y| {
            let Some(this) = weak.upgrade() else { return; };
            let state_guard = this.imp().get_state();
            if let Some(state_rc) = state_guard.as_ref() {
                *pan_at_start.borrow_mut() = state_rc.borrow().viewer.pan_offset;
            }
        });

        let weak = self.downgrade();
        drag.connect_drag_update(move |_, dx, dy| {
            let Some(this) = weak.upgrade() else { return; };
            let base = *pan_at_start2.borrow();
            {
                let state_guard = this.imp().get_state();
                if let Some(state_rc) = state_guard.as_ref() {
                    let mut st = state_rc.borrow_mut();
                    st.viewer.pan_offset = (base.0 + dx, base.1 + dy);
                    st.viewer.clamp_pan();
                }
            }
            this.queue_draw();
        });

        self.add_controller(drag);
    }
}
