// Ui
use ui::Ui;

// Standard
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::f32::consts::PI;
use std::collections::HashMap;
use std::cell::RefCell;
//use std::f32::{sin, cos};

// Library
use nalgebra::{Vector2, Vector3, Translation3, Rotation3, convert, dot};
use coord::prelude::*;
use glutin::{ElementState, VirtualKeyCode};
use dot_vox;

// Project
use client;
use client::{Client, ClientMode};
use client::CHUNK_SIZE;
use region::{Chunk, VolState};

// Local
use camera::Camera;
use window::{RenderWindow, Event};
use model_object::{ModelObject, Constants};
use mesh::{Mesh};
use keybinds::Keybinds;
use key_state::KeyState;
use vox::vox_to_model;

pub struct Payloads {}
impl client::Payloads for Payloads {
    type Chunk = (Mesh, Option<ModelObject>);
}

pub struct Game {
    running: AtomicBool,
    client: Arc<Client<Payloads>>,
    window: RenderWindow,
    data: Mutex<Data>,
    camera: Mutex<Camera>,
    key_state: Mutex<KeyState>,
    ui: RefCell<Ui>,
    keys: Keybinds,
}

// "Data" includes mutable state
struct Data {
    player_model: ModelObject,
    other_player_model: ModelObject,
}

fn gen_payload(chunk: &Chunk) -> <Payloads as client::Payloads>::Chunk {
    (Mesh::from(chunk), None)
}

impl Game {
    pub fn new<R: ToSocketAddrs>(mode: ClientMode, alias: &str, remote_addr: R, view_distance: i64) -> Game {
        let window = RenderWindow::new();

        info!("trying to load model files");
        let vox = dot_vox::load("data/vox/3.vox")
            .expect("cannot find model 3.vox. Make sure to start voxygen from its folder");
        let voxmodel = vox_to_model(vox);

        let player_mesh = Mesh::from_with_offset(&voxmodel, vec3!(-10.0, -4.0, 0.0));

        let player_model = ModelObject::new(
            &mut window.renderer_mut(),
            &player_mesh,
        );

        let vox = dot_vox::load("data/vox/5.vox")
            .expect("cannot find model 5.vox. Make sure to start voxygen from its folder");
        let voxmodel = vox_to_model(vox);

        let other_player_mesh = Mesh::from(&voxmodel);

        let other_player_model = ModelObject::new(
            &mut window.renderer_mut(),
            &other_player_mesh,
        );

        let client = Client::new(mode, alias.to_string(), remote_addr, gen_payload, view_distance)
            .expect("Could not create new client");

        // Contruct the UI
        let window_dims = window.get_size();

        let mut ui = Ui::new(&mut window.renderer_mut(), window_dims, &client);

        client.start();

        Game {
            data: Mutex::new(Data {
                player_model,
                other_player_model,
            }),
            running: AtomicBool::new(true),
            client,
            window,
            camera: Mutex::new(Camera::new()),
            key_state: Mutex::new(KeyState::new()),
            ui: RefCell::new(ui),
            keys: Keybinds::new(),
        }
    }

    pub fn handle_window_events(&self) -> bool {
        self.window.handle_events(|event| {
            match event {
                Event::CloseRequest => self.running.store(false, Ordering::Relaxed),
                Event::CursorMoved { dx, dy } => {
                    let data = self.data.lock().unwrap();

                    if self.window.cursor_trapped().load(Ordering::Relaxed) {
                        //debug!("dx: {}, dy: {}", dx, dy);
                        self.camera.lock().unwrap().rotate_by(Vector2::new(dx as f32 * 0.002, dy as f32 * 0.002));
                    }
                },
                Event::MouseWheel { dy, .. } => {
                    self.camera.lock().unwrap().zoom_by((-dy / 4.0) as f32);
                },
                Event::KeyboardInput { i, .. } => {
                    // Helper function to determine scancode equality
                    fn keypress_eq(key: &Option<u32>, scancode: u32) -> bool {
                        key.map(|sc| sc == scancode).unwrap_or(false)
                    }

                    // Helper variables to clean up code. Add any new input modes here.
                    let general = &self.keys.general;
                    let mount = &self.keys.mount;
                    let show_chat = self.ui.borrow().get_show_chat();

                    // General inputs -------------------------------------------------------------
                    if keypress_eq(&general.pause, i.scancode) { // Default: Escape (free cursor)
                        self.window.untrap_cursor();
                    } else if keypress_eq(&general.use_item, i.scancode) { // Default: Ctrl+Q (quit) (temporary)
                        if i.modifiers.ctrl {
                            self.running.store(false, Ordering::Relaxed);
                        }
                    } else if keypress_eq(&general.chat, i.scancode) && i.state == ElementState::Released {
                        self.ui.borrow_mut().set_show_chat(!show_chat);
                    }

                    if !show_chat {
                        if keypress_eq(&general.forward, i.scancode) {
                            self.key_state.lock().unwrap().up = match i.state { // Default: W (up)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.left, i.scancode) {
                            self.key_state.lock().unwrap().left = match i.state { // Default: A (left)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.back, i.scancode) {
                            self.key_state.lock().unwrap().down = match i.state { // Default: S (down)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.right, i.scancode) {
                            self.key_state.lock().unwrap().right = match i.state { // Default: D (right)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.fly, i.scancode) {
                            self.key_state.lock().unwrap().fly = match i.state { // Default: Space (fly)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.fall, i.scancode) {
                            self.key_state.lock().unwrap().fall = match i.state { // Default: Shift (fall)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        }
                    }

                    // ----------------------------------------------------------------------------

                    // Mount inputs ---------------------------------------------------------------
                    // placeholder
                    // ----------------------------------------------------------------------------
                },
                Event::Raw { event } => {
                    self.ui.borrow_mut().handle_event(event);
                },
                _ => { },
            }
        });

        loop {
            let mut inp = String::new();
            if let Ok(_) = io::std().read_line(&mut inp) {
                match inp {
                    "PLAY" => println!("You typed play!"),
                    "OPTIONS" => println!("You typed options!"),
                    "PLAY" => { println!("You typed exit!"); break; },
                    _ => println!("Unrecognised input!"),
                }
            }
        }

        // Calculate movement player movement vector
        let ori = *self.camera.lock().unwrap().ori();
        let unit_vecs = (
            Vector2::new(ori.x.cos(), -ori.x.sin()),
            Vector2::new(ori.x.sin(), ori.x.cos())
        );
        let dir_vec = self.key_state.lock().unwrap().dir_vec();
        let mov_vec = unit_vecs.0 * dir_vec.x + unit_vecs.1 * dir_vec.y;
        let fly_vec = self.key_state.lock().unwrap().fly_vec();

        // Why do we do this in Voxygen?!
        if let Some(player_entity) = self.client.player_entity() {
            let mut player_entity = player_entity.write().unwrap();

            // Apply acceleration
            player_entity.ctrl_vel_mut().x += mov_vec.x * 0.2;
            player_entity.ctrl_vel_mut().y += mov_vec.y * 0.2;

            // Apply friction
            player_entity.ctrl_vel_mut().x *= 0.85;
            player_entity.ctrl_vel_mut().y *= 0.85;

            // Apply jumping
            *player_entity.jumping_mut() = self.key_state.lock().unwrap().jumping();

            let vel = *player_entity.ctrl_vel_mut();
            let ori = *self.camera.lock().unwrap().ori();

            // Apply rotating
            if vel.length() > 0.5 {
                player_entity.look_dir_mut().x = vel.x.atan2(vel.y);
            }

            // Apply leaning
            player_entity.look_dir_mut().y = vec2!(vel.x, vel.y).length() * 0.3;
        }

        // Set camera focus to the player's head
        if let Some(player_entity) = self.client.player_entity() {
            let player_entity = player_entity.read().unwrap();
            self.camera.lock().unwrap().set_focus(Vector3::<f32>::new(
                player_entity.pos().x,
                player_entity.pos().y,
                player_entity.pos().z + 1.75
            ));
        }

        self.running.load(Ordering::Relaxed)
    }

    pub fn model_chunks(&self) {
        for (pos, vol) in self.client.chunk_mgr().volumes().iter() {
            if let VolState::Exists(ref chunk, ref mut payload) = *vol.write().unwrap() {
                if let None = payload.1 {
                    payload.1 = Some(ModelObject::new(
                        &mut self.window.renderer_mut(),
                        &payload.0,
                    ));
                }
            }
        }
    }

    pub fn render_frame(&self) {
        let mut renderer = self.window.renderer_mut();
        renderer.begin_frame();

        let camera_mats = self.camera.lock().unwrap().get_mats();
        let camera_ori = self.camera.lock().unwrap().ori();

        for (pos, vol) in self.client.chunk_mgr().volumes().iter() {
            if let VolState::Exists(ref chunk, ref payload) = *vol.read().unwrap() {
                if let Some(ref model) = payload.1 {
                    let model_mat = &Translation3::<f32>::from_vector(Vector3::<f32>::new(
                        (pos.x * CHUNK_SIZE) as f32,
                        (pos.y * CHUNK_SIZE) as f32,
                        0.0
                    )).to_homogeneous();

                    renderer.update_model_object(
                        &model,
                        Constants::new(
                            &model_mat, // TODO: Improve this
                            &camera_mats.0,
                            &camera_mats.1,
                        )
                    );
                    renderer.render_model_object(&model);
                }
            }
        }

        // Render each and every entity
        for (uid, entity) in self.client.entities().iter() {
            let entity = entity.read().unwrap();

            // Calculate a transformation matrix for the entity's model
            let model_mat = &Translation3::from_vector(Vector3::new(
                    entity.pos().x,
                    entity.pos().y,
                    entity.pos().z
                )).to_homogeneous()
                * Rotation3::new(Vector3::new(0.0, 0.0, PI - entity.look_dir().x)).to_homogeneous()
                * Rotation3::new(Vector3::new(entity.look_dir().y, 0.0, 0.0)).to_homogeneous();

            // Choose the correct model for the entity
            let mut data = self.data.lock().unwrap();
            let model = match self.client.player().entity_uid {
                Some(uid) if uid == uid => &data.player_model,
                _ => &data.other_player_model,
            };

            // Update the model's constant buffer with the transformation details previously calculated
            renderer.update_model_object(
                &model,
                // TODO: Improve this
                Constants::new(
                    &model_mat,
                    &camera_mats.0,
                    &camera_mats.1,
                )
            );

            // Actually render the model
            renderer.render_model_object(&model);
        }

        // Draw ui
        self.ui.borrow_mut().render(&mut renderer, &self.client.clone(), &self.window.get_size());

        self.window.swap_buffers();
        renderer.end_frame();
    }

    pub fn run(&self) {
        while self.handle_window_events() {
            self.model_chunks();
            self.render_frame();
        }

		self.client.shutdown();
    }
}
