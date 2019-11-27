#![forbid(unsafe_code)]

mod fps;
mod world;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    web_sys::window()
        .unwrap_throw()
        .request_animation_frame(&State::new().0.borrow().animation_frame_closure)
        .unwrap_throw();
}

#[derive(Clone)]
struct State(Rc<RefCell<Model>>);

struct Model {
    animation_frame_closure: js_sys::Function,
    keys: HashSet<String>,
    fps: Option<fps::FrameCounter>,
    vr_status: VrStatus,

    world: world::World,

    window: web_sys::Window,
    document: web_sys::Document,
    canvas: web_sys::HtmlCanvasElement,
    info_box: web_sys::HtmlParagraphElement,

    player: Player,
}

struct Player {
    position: nalgebra::Vector4<f32>,
    // World space to player space.
    horizontal_orientation: nalgebra::UnitQuaternion<f32>,
    // Angle above horizontal.
    vertical_angle: f32,
}

enum Msg {
    Click,
    MouseMove([i32; 2]),
    MouseWheel(f64),
    KeyDown(String),
    KeyUp(String),

    GotVRDisplays(js_sys::Array),
    DisplayPresenting(web_sys::VrDisplay),
}

enum VrStatus {
    Searching,
    NotFound {
        three_camera_rot: nalgebra::UnitQuaternion<f32>,
    },
    Known(web_sys::VrDisplay),
    RequestedPresentation(web_sys::VrDisplay),
    Presenting(web_sys::VrDisplay),
}

impl State {
    fn new() -> Self {
        let out = Self(Rc::new(RefCell::new(Model::new())));

        {
            let model: &mut Model = &mut out.0.borrow_mut();

            let navigator: web_sys::Navigator = model.window.navigator();
            if js_sys::Reflect::has(&navigator, &"getVRDisplays".into()).unwrap_throw() {
                let state = out.clone();
                let closure = Closure::once(move |vr_displays| {
                    state.update(Msg::GotVRDisplays(js_sys::Array::from(&vr_displays)));
                });
                navigator.get_vr_displays().unwrap_throw().then(&closure);
                closure.forget();
            } else {
                web_sys::console::error_1(
                    &"WebVR is not supported by this browser, on this computer.".into(),
                );

                model.vr_status = VrStatus::NotFound {
                    three_camera_rot: nalgebra::UnitQuaternion::identity(),
                };
            }

            out.event_listener(&model.canvas, "mousedown", |_| Msg::Click);
            out.event_listener(&model.canvas, "mousemove", |evt| {
                let evt = evt.dyn_into::<web_sys::MouseEvent>().unwrap_throw();
                Msg::MouseMove([evt.movement_x(), evt.movement_y()])
            });
            out.event_listener(&model.canvas, "wheel", |evt| {
                let evt = evt.dyn_into::<web_sys::WheelEvent>().unwrap_throw();
                Msg::MouseWheel(evt.delta_y())
            });
            out.event_listener(&model.document, "keydown", |evt| {
                let evt = evt.dyn_into::<web_sys::KeyboardEvent>().unwrap_throw();
                Msg::KeyDown(evt.key())
            });
            out.event_listener(&model.document, "keyup", |evt| {
                let evt = evt.dyn_into::<web_sys::KeyboardEvent>().unwrap_throw();
                Msg::KeyUp(evt.key())
            });

            let state = out.clone();
            let closure: Closure<dyn FnMut(f64)> = Closure::wrap(Box::new(move |timestamp| {
                state.frame(timestamp);
            }));
            model.animation_frame_closure =
                closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
            closure.forget();
        }

        out
    }

    fn update(&self, msg: Msg) {
        let model: &mut Model = &mut self.0.borrow_mut();

        match msg {
            Msg::Click => {
                if model.document.pointer_lock_element().is_none() {
                    model.canvas.request_pointer_lock();
                }
                if let VrStatus::Known(display) = &model.vr_status {
                    let mut layer = web_sys::VrLayer::new();
                    layer.source(Some(&model.canvas));
                    let layers = js_sys::Array::new();
                    layers.set(0, layer.as_ref().clone());

                    let state = self.clone();
                    let display_ = display.clone();
                    let closure =
                        Closure::once(move |_| state.update(Msg::DisplayPresenting(display_)));
                    display
                        .request_present(&layers)
                        .unwrap_throw()
                        .then(&closure);
                    closure.forget();

                    model.vr_status = VrStatus::RequestedPresentation(display.clone());
                }

                let cast_result =
                    model
                        .world
                        .raycast(model.player.position, model.player.direction(), 5.);

                web_sys::console::log_1(&format!("{:?}", cast_result).into());
                if let (Some(block), _) = cast_result {
                    model.world.set_block(block, Some([0x80, 0x20, 0x20]));
                }
            }
            Msg::KeyDown(k) => {
                model.keys.insert(k.to_lowercase());
            }
            Msg::KeyUp(k) => {
                model.keys.remove(&k.to_lowercase());
            }
            Msg::MouseMove([x, y]) => {
                if model.document.pointer_lock_element().is_some() {
                    model.player.horizontal_orientation = nalgebra::UnitQuaternion::new(
                        nalgebra::Vector3::new(y as f32 * 3e-3, -x as f32 * 3e-3, 0.),
                    ) * model.player.horizontal_orientation;
                }
            }
            Msg::MouseWheel(y) => {
                if model.document.pointer_lock_element().is_some() {
                    model.player.vertical_angle = (model.player.vertical_angle + 0.1 * y as f32)
                        .max(-std::f32::consts::FRAC_PI_2)
                        .min(std::f32::consts::FRAC_PI_2);
                }
            }

            Msg::GotVRDisplays(vr_displays) => {
                if vr_displays.length() == 0 {
                    model.vr_status = VrStatus::NotFound {
                        three_camera_rot: nalgebra::UnitQuaternion::identity(),
                    };
                } else {
                    model.vr_status = VrStatus::Known(vr_displays.get(0).dyn_into().unwrap_throw());
                }
            }
            Msg::DisplayPresenting(display) => model.vr_status = VrStatus::Presenting(display),
        }
    }

    fn frame(&self, timestamp: f64) {
        let model: &mut Model = &mut self.0.borrow_mut();

        if let VrStatus::Presenting(display) = &model.vr_status {
            display
                .request_animation_frame(&model.animation_frame_closure)
                .unwrap_throw();
        } else {
            model
                .window
                .request_animation_frame(&model.animation_frame_closure)
                .unwrap_throw();
        }

        if let Some(fps) = &mut model.fps {
            let dt = fps.frame(timestamp);
            model.info_box.set_inner_text(&format!(
                "{}\n\n{:?}",
                fps,
                model.player.position.as_slice()
            ));

            {
                let mut movement_vector = nalgebra::Vector4::zeros();
                if model.keys.contains(" ") {
                    movement_vector += nalgebra::Vector4::w();
                }
                if model.keys.contains("shift") {
                    movement_vector -= nalgebra::Vector4::w();
                }
                if model.keys.contains("w") {
                    movement_vector += nalgebra::Vector4::z();
                }
                if model.keys.contains("s") {
                    movement_vector -= nalgebra::Vector4::z();
                }
                if model.keys.contains("a") {
                    movement_vector -= nalgebra::Vector4::x();
                }
                if model.keys.contains("d") {
                    movement_vector += nalgebra::Vector4::x();
                }
                if model.keys.contains("q") {
                    movement_vector += nalgebra::Vector4::y();
                }
                if model.keys.contains("e") {
                    movement_vector -= nalgebra::Vector4::y();
                }
                model
                    .player
                    .r#move(movement_vector * dt as f32, &model.world);
            }

            if let VrStatus::NotFound { three_camera_rot } = &mut model.vr_status {
                if model.keys.contains("arrowdown") {
                    *three_camera_rot =
                        nalgebra::UnitQuaternion::new(nalgebra::Vector3::new(-0.01, 0.0, 0.0f32))
                            * *three_camera_rot;
                }
                if model.keys.contains("arrowup") {
                    *three_camera_rot =
                        nalgebra::UnitQuaternion::new(nalgebra::Vector3::new(0.01, 0.0, 0.0f32))
                            * *three_camera_rot;
                }
                if model.keys.contains("arrowright") {
                    *three_camera_rot =
                        nalgebra::UnitQuaternion::new(nalgebra::Vector3::new(0.0, -0.01, 0.0f32))
                            * *three_camera_rot;
                }
                if model.keys.contains("arrowleft") {
                    *three_camera_rot =
                        nalgebra::UnitQuaternion::new(nalgebra::Vector3::new(0.0, 0.01, 0.0f32))
                            * *three_camera_rot;
                }
            }

            {
                if let VrStatus::Presenting(display) = &model.vr_status {
                    let frame_data = web_sys::VrFrameData::new().unwrap_throw();
                    display.get_frame_data(&frame_data);

                    let views = vec![
                        world::View {
                            three_camera: nalgebra::MatrixSlice4::from_slice(
                                &frame_data.left_projection_matrix().unwrap_throw(),
                            ) * nalgebra::MatrixSlice4::from_slice(
                                &frame_data.left_view_matrix().unwrap_throw(),
                            ),
                            viewport_start: [0, 0],
                            viewport_size: [
                                model.canvas.width() as i32 / 2,
                                model.canvas.height() as i32,
                            ],
                        },
                        world::View {
                            three_camera: nalgebra::MatrixSlice4::from_slice(
                                &frame_data.right_projection_matrix().unwrap_throw(),
                            ) * nalgebra::MatrixSlice4::from_slice(
                                &frame_data.right_view_matrix().unwrap_throw(),
                            ),
                            viewport_start: [model.canvas.width() as i32 / 2, 0],
                            viewport_size: [
                                model.canvas.width() as i32 / 2,
                                model.canvas.height() as i32,
                            ],
                        },
                    ];
                    model.world.render(model.player.projection_matrix(), views);
                } else if let VrStatus::NotFound { three_camera_rot } = &model.vr_status {
                    let views = vec![world::View {
                        three_camera: nalgebra::Matrix4::new(
                            1.,
                            0.,
                            0.,
                            0.,
                            0.,
                            model.canvas.width() as f32 / model.canvas.height() as f32,
                            0.,
                            0.,
                            0.,
                            0.,
                            1.,
                            -2.98,
                            0.,
                            0.,
                            -1.,
                            3.,
                        ) * three_camera_rot.to_homogeneous(),
                        viewport_start: [0, 0],
                        viewport_size: [model.canvas.width() as i32, model.canvas.height() as i32],
                    }];
                    model.world.render(model.player.projection_matrix(), views);
                };

                if let VrStatus::Presenting(display) = &model.vr_status {
                    display.submit_frame();
                }
            }
        } else {
            model.fps = Some(fps::FrameCounter::new(timestamp))
        }
    }

    fn event_listener(
        &self,
        target: &web_sys::EventTarget,
        event: &str,
        msg: impl Fn(web_sys::Event) -> Msg + 'static,
    ) {
        let state = self.clone();
        let closure: Closure<dyn FnMut(web_sys::Event)> = Closure::wrap(Box::new(move |evt| {
            state.update(msg(evt));
        }));
        target
            .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
            .unwrap_throw();
        closure.forget();
    }
}

impl Model {
    fn new() -> Self {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let body = document.body().unwrap_throw();

        let canvas = document
            .create_element("canvas")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap_throw();
        canvas.set_attribute("width", "1600").unwrap_throw();
        canvas.set_attribute("height", "800").unwrap_throw();
        body.append_child(&canvas).unwrap_throw();

        let info_box = document
            .create_element("p")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlParagraphElement>()
            .unwrap_throw();
        body.append_child(&info_box).unwrap_throw();

        let gl = canvas
            .get_context("webgl2")
            .unwrap_throw()
            .unwrap_throw()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap_throw();
        let mut world = world::World::new(gl);

        // for i in 0..=3 {
        //     for j in 0..=3 {
        //         for k in 0..=3 {
        //             for l in 0..=2 {
        //                 world.set_block(
        //                     [i, j, k, l],
        //                     Some([i as u8 * 0x3F, j as u8 * 0x3F, k as u8 * 0x3F]),
        //                 )
        //             }
        //         }
        //     }
        // }
        // world.set_block([1, 1, 2, 1], None);
        // world.set_block([1, 2, 2, 1], None);
        // world.set_block([1, 2, 1, 1], None);
        // world.set_block([2, 2, 1, 1], None);
        // world.set_block([2, 1, 1, 1], None);
        // world.set_block([2, 1, 2, 1], None);

        for i in 0..world::WORLD_SIZE {
            for j in 0..world::WORLD_SIZE {
                for k in 0..world::WORLD_SIZE {
                    world.set_block([i as i32, j as i32, k as i32, 0], Some([0x50, 0xC0, 0x50]))
                }
            }
        }

        let mut tree = |[x, y, z]: [i32; 3]| {
            for dx in -2..=2 {
                for dy in -2..=2 {
                    for dz in -2..=2 {
                        if dx * dx + dy * dy + dz * dz < 7 {
                            world.set_block([x + dx, y + dy, z + dz, 3], Some([0x20, 0xC0, 0x20]));
                            world.set_block([x + dx, y + dy, z + dz, 4], Some([0x20, 0xC0, 0x20]));
                        }
                        if dx * dx + dy * dy + dz * dz < 3 {
                            world.set_block([x + dx, y + dy, z + dz, 5], Some([0x20, 0xC0, 0x20]));
                            world.set_block([x + dx, y + dy, z + dz, 6], Some([0x20, 0xC0, 0x20]));
                        }
                    }
                }
            }
            for w in 1..=5 {
                world.set_block([x, y, z, w], Some([0x80, 0x40, 0x00]));
            }
        };

        tree([2, 2, 2]);
        tree([5, 5, 5]);

        Self {
            animation_frame_closure: JsValue::undefined().into(),
            fps: None,
            keys: HashSet::new(),
            vr_status: VrStatus::Searching,
            world,

            window,
            document,
            canvas,
            info_box,

            player: Player::new(),
        }
    }
}

impl Player {
    fn new() -> Self {
        Self {
            position: nalgebra::Vector4::new(2.5, 1.5, 1.5, 1.5),
            horizontal_orientation: nalgebra::UnitQuaternion::identity(),
            vertical_angle: 0.0,
        }
    }

    // Direction is relative to player.
    fn r#move(&mut self, mut direction: nalgebra::Vector4<f32>, world: &world::World) {
        let mut horiz = direction.fixed_rows_mut::<nalgebra::U3>(0);
        horiz.copy_from(&(self.horizontal_orientation.conjugate() * horiz.clone_owned()));

        // direction now in world coordinates

        let new_position = self.position + direction;

        let mut new_position_integer = [0; 4];
        for i in 0..4 {
            new_position_integer[i] = new_position[i].floor() as i32;
        }
        if world.get_block(new_position_integer).is_none() {
            self.position = new_position;
        }
    }

    fn rotation_matrix(&self) -> nalgebra::Matrix4<f32> {
        let vertical_rotation: nalgebra::Matrix4<f32> = {
            let (s, c) = self.vertical_angle.sin_cos();
            let mut out = nalgebra::Matrix4::identity();
            out[(2, 2)] = c;
            out[(2, 3)] = s;
            out[(3, 2)] = -s;
            out[(3, 3)] = c;
            out
        };

        let horizontal_rotation: nalgebra::Matrix4<f32> =
            self.horizontal_orientation.to_homogeneous();

        vertical_rotation * horizontal_rotation
    }

    fn projection_matrix(&self) -> nalgebra::Matrix5<f32> {
        // Project to screen-depth space, with y = up, w = depth coordinate, v = homogeneous coordinate. Infinity projects to w=0.
        #[rustfmt::skip]
        let projection_matrix = nalgebra::Matrix5::new(
            1., 0., 0., 0., 0.,
            0., 0., 0., 1., 0.,
            0., 1., 0., 0., 0.,
            0., 0., 0., 0., -1.,
            0., 0., 1., 0., 0.,
        );

        // move everything so that camera is at origin.
        let translation: nalgebra::Matrix5<f32> =
            nalgebra::Translation::from(-self.position).to_homogeneous();

        projection_matrix * self.rotation_matrix().to_homogeneous() * translation
    }

    fn direction(&self) -> nalgebra::Vector4<f32> {
        self.rotation_matrix().try_inverse().unwrap_throw() * nalgebra::Vector4::new(0., 0., 1., 0.)
    }
}
