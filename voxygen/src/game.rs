// Ui
use ui::Ui;

// Standard
use std::{
    cell::RefCell,
    f32::consts::PI,
    net::ToSocketAddrs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
//use std::f32::{sin, cos};

// Library
use coord::prelude::*;
use dot_vox;
use glutin::ElementState;
use nalgebra::{Rotation3, Translation3, Vector2, Vector3};

// Project
use client::{self, Client, ClientMode, CHUNK_SIZE};
use region::{Chunk, VolState};

// Local
use camera::Camera;
use consts::{ConstHandle, GlobalConsts};
use key_state::KeyState;
use keybinds::Keybinds;
use pipeline::Pipeline;
use shader::Shader;
use skybox;
use tonemapper;
use voxel;
use window::{Event, RenderWindow};

pub enum ChunkPayload {
    Mesh(voxel::Mesh),
    Model(voxel::Model),
}

pub struct Payloads {}
impl client::Payloads for Payloads {
    type Chunk = ChunkPayload;
}

pub struct Game {
    running: AtomicBool,

    client: Arc<Client<Payloads>>,
    window: RenderWindow,

    global_consts: ConstHandle<GlobalConsts>,
    camera: Mutex<Camera>,

    ui: RefCell<Ui>,

    key_state: Mutex<KeyState>,
    keys: Keybinds,

    skybox_pipeline: Pipeline<skybox::pipeline::Init<'static>>,
    voxel_pipeline: Pipeline<voxel::pipeline::Init<'static>>,
    tonemapper_pipeline: Pipeline<tonemapper::pipeline::Init<'static>>,

    skybox_model: skybox::Model,
    player_model: voxel::Model,
    other_player_model: voxel::Model,
}

fn gen_payload(chunk: &Chunk) -> <Payloads as client::Payloads>::Chunk { ChunkPayload::Mesh(voxel::Mesh::from(chunk)) }

impl Game {
    pub fn new<R: ToSocketAddrs>(mode: ClientMode, alias: &str, remote_addr: R, view_distance: i64) -> Game {
        let window = RenderWindow::new();

        let global_consts = ConstHandle::new(&mut window.renderer_mut());

        let skybox_mesh = skybox::Mesh::new_skybox();
        let skybox_model = skybox::Model::new(&mut window.renderer_mut(), &skybox_mesh);

        info!("trying to load model files");
        let vox = dot_vox::load("assets/cosmetic/creature/friendly/player3.vox")
            .expect("cannot find model 3.vox. Make sure to start voxygen from its folder");
        let voxmodel = voxel::vox_to_figure(vox);

        let player_mesh = voxel::Mesh::from_with_offset(&voxmodel, vec3!(-10.0, -4.0, 0.0));

        let player_model = voxel::Model::new(&mut window.renderer_mut(), &player_mesh);

        let vox = dot_vox::load("assets/cosmetic/creature/friendly/player5.vox")
            .expect("cannot find model 5.vox. Make sure to start voxygen from its folder");
        let voxmodel = voxel::vox_to_figure(vox);

        let other_player_mesh = voxel::Mesh::from(&voxmodel);

        let other_player_model = voxel::Model::new(&mut window.renderer_mut(), &other_player_mesh);

        let client = Client::new(mode, alias.to_string(), remote_addr, gen_payload, view_distance)
            .expect("Could not create new client");
        client.start();

        // Contruct the UI
        let window_dims = window.get_size();
        let ui = Ui::new(&mut window.renderer_mut(), window_dims, &client);

        // Create pipelines

        let voxel_pipeline = Pipeline::new(
            window.renderer_mut().factory_mut(),
            voxel::pipeline::new(),
            &Shader::from_file("shaders/voxel/vert.glsl").expect("Could not load voxel vertex shader"),
            &Shader::from_file("shaders/voxel/frag.glsl").expect("Could not load voxel fragment shader"),
        );

        let skybox_pipeline = Pipeline::new(
            window.renderer_mut().factory_mut(),
            skybox::pipeline::new(),
            &Shader::from_file("shaders/skybox/vert.glsl").expect("Could not load skybox vertex shader"),
            &Shader::from_file("shaders/skybox/frag.glsl").expect("Could not load skybox fragment shader"),
        );

        let tonemapper_pipeline = Pipeline::new(
            window.renderer_mut().factory_mut(),
            tonemapper::pipeline::new(),
            &Shader::from_file("shaders/tonemapper/vert.glsl").expect("Could not load skybox vertex shader"),
            &Shader::from_file("shaders/tonemapper/frag.glsl").expect("Could not load skybox fragment shader"),
        );

        Game {
            running: AtomicBool::new(true),

            client,
            window,

            global_consts,
            camera: Mutex::new(Camera::new()),

            ui: RefCell::new(ui),

            key_state: Mutex::new(KeyState::new()),
            keys: Keybinds::new(),

            skybox_pipeline,
            voxel_pipeline,
            tonemapper_pipeline,

            skybox_model,
            player_model,
            other_player_model,
        }
    }

    pub fn handle_window_events(&self) -> bool {
        self.window.handle_events(|event| {
            match event {
                Event::CloseRequest => self.running.store(false, Ordering::Relaxed),
                Event::CursorMoved { dx, dy } => {
                    if self.window.cursor_trapped().load(Ordering::Relaxed) {
                        self.camera
                            .lock()
                            .unwrap()
                            .rotate_by(Vector2::new(dx as f32 * 0.002, dy as f32 * 0.002));
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
                    let show_chat = self.ui.borrow().get_show_chat();

                    // General inputs -------------------------------------------------------------
                    if keypress_eq(&general.pause, i.scancode) {
                        // Default: Escape (free cursor)
                        self.window.untrap_cursor();
                    } else if keypress_eq(&general.use_item, i.scancode) {
                        // Default: Ctrl+Q (quit) (temporary)
                        if i.modifiers.ctrl {
                            self.running.store(false, Ordering::Relaxed);
                        }
                    } else if keypress_eq(&general.chat, i.scancode) && i.state == ElementState::Released {
                        self.ui.borrow_mut().set_show_chat(!show_chat);
                    }

                    if !show_chat {
                        if keypress_eq(&general.forward, i.scancode) {
                            self.key_state.lock().unwrap().up = match i.state {
                                // Default: W (up)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.left, i.scancode) {
                            self.key_state.lock().unwrap().left = match i.state {
                                // Default: A (left)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.back, i.scancode) {
                            self.key_state.lock().unwrap().down = match i.state {
                                // Default: S (down)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.right, i.scancode) {
                            self.key_state.lock().unwrap().right = match i.state {
                                // Default: D (right)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.jump, i.scancode) {
                            self.key_state.lock().unwrap().jump = match i.state {
                                // Default: Space (fly)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.crouch, i.scancode) {
                            // self.key_state.lock().unwrap().fall = match i.state { // Default: Shift (fall)
                            //     ElementState::Pressed => true,
                            //     ElementState::Released => false,
                            // }
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
                Event::Resized { w, h } => {
                    self.camera
                        .lock()
                        .unwrap()
                        .set_aspect_ratio((w.max(1) as f32) / (h.max(1) as f32));
                },
                _ => {},
            }
        });

        // Calculate movement player movement vector
        let ori = *self.camera.lock().unwrap().ori();
        let unit_vecs = (
            Vector2::new(ori.x.cos(), -ori.x.sin()),
            Vector2::new(ori.x.sin(), ori.x.cos()),
        );
        let dir_vec = self.key_state.lock().unwrap().dir_vec();
        let mov_vec = unit_vecs.0 * dir_vec.x + unit_vecs.1 * dir_vec.y;

        // Why do we do this in Voxygen?!
        const LOOKING_VEL_FAC: f32 = 1.0;
        const LOOKING_CTRL_ACC_FAC: f32 = 1.0;
        const MIN_LOOKING: f32 = 0.5;
        const LEANING_FAC: f32 = 0.05;
        if let Some(player_entity) = self.client.player_entity() {
            let mut player_entity = player_entity.write().unwrap();

            // Apply acceleration
            player_entity.ctrl_acc_mut().x = mov_vec.x;
            player_entity.ctrl_acc_mut().y = mov_vec.y;

            // Apply jumping
            player_entity.ctrl_acc_mut().z = if self.key_state.lock().unwrap().jump() {
                1.0
            } else {
                0.0
            };

            let looking = (*player_entity.vel() * LOOKING_VEL_FAC
                + *player_entity.ctrl_acc_mut() * LOOKING_CTRL_ACC_FAC)
                / (LOOKING_VEL_FAC + LOOKING_CTRL_ACC_FAC);

            // Apply rotating
            if looking.length() > MIN_LOOKING {
                player_entity.look_dir_mut().x = looking.x.atan2(looking.y);
            }

            // Apply leaning
            player_entity.look_dir_mut().y = vec2!(looking.x, looking.y).length() * LEANING_FAC;
        }

        // Set camera focus to the player's head
        if let Some(player_entity) = self.client.player_entity() {
            let player_entity = player_entity.read().unwrap();
            self.camera.lock().unwrap().set_focus(Vector3::<f32>::new(
                player_entity.pos().x,
                player_entity.pos().y,
                player_entity.pos().z + 1.75,
            ));
        }

        self.running.load(Ordering::Relaxed)
    }

    pub fn model_chunks(&self) {
        for (_, vol) in self.client.chunk_mgr().volumes().iter() {
            if let VolState::Exists(_, ref mut payload) = *vol.write().unwrap() {
                if let ChunkPayload::Mesh(ref mut mesh) = payload {
                    *payload = ChunkPayload::Model(voxel::Model::new(&mut self.window.renderer_mut(), mesh));
                }
            }
        }
    }

    pub fn render_frame(&self) {
        // Calculate frame constants
        let camera_mats = self.camera.lock().unwrap().get_mats();
        let cam_origin = *self.camera.lock().unwrap().get_pos();
        let play_origin = self
            .client
            .player_entity()
            .map(|p| *p.read().unwrap().pos())
            .unwrap_or(vec3!(0.0, 0.0, 0.0));
        let play_origin = [play_origin.x, play_origin.y, play_origin.z, 1.0];
        let time = self.client.time() as f32;

        // Begin rendering, don't clear the frame
        let mut renderer = self.window.renderer_mut();
        renderer.begin_frame(None);

        // Update global constants that apply to the entire frame
        self.global_consts.update(
            &mut renderer,
            GlobalConsts {
                view_mat: *camera_mats.0.as_ref(),
                proj_mat: *camera_mats.1.as_ref(),
                cam_origin: [cam_origin.x, cam_origin.y, cam_origin.z, 1.0],
                play_origin,
                view_distance: [self.client.view_distance(); 4],
                time: [time; 4],
            },
        );

        // Render the skybox
        self.skybox_model
            .render(&mut renderer, &self.skybox_pipeline, &self.global_consts);

        // Render each chunk
        for (pos, vol) in self.client.chunk_mgr().volumes().iter() {
            if let VolState::Exists(ref chunk, ref payload) = *vol.read().unwrap() {
                if let ChunkPayload::Model(ref model) = payload {
                    let model_mat = &Translation3::<f32>::from_vector(Vector3::<f32>::new(
                        (pos.x * CHUNK_SIZE) as f32,
                        (pos.y * CHUNK_SIZE) as f32,
                        0.0,
                    )).to_homogeneous();

                    // TODO: We don't need to update this *every* frame. Be cleverer about this.
                    model.const_handle().update(
                        &mut renderer,
                        voxel::ModelConsts {
                            model_mat: *model_mat.as_ref(),
                        },
                    );

                    model.render(&mut renderer, &self.voxel_pipeline, &self.global_consts);
                }
            }
        }

        // Render each entity
        for (uid, entity) in self.client.entities().iter() {
            let entity = entity.read().unwrap();

            // Calculate a transformation matrix for the entity's model
            let model_mat = &Translation3::from_vector(Vector3::from(entity.pos().elements())).to_homogeneous()
                * Rotation3::new(Vector3::new(0.0, 0.0, PI - entity.look_dir().x)).to_homogeneous()
                * Rotation3::new(Vector3::new(entity.look_dir().y, 0.0, 0.0)).to_homogeneous();

            // Choose the correct model for the entity
            let model = match self.client.player().entity_uid {
                Some(uid) if uid == uid => &self.player_model,
                _ => &self.other_player_model,
            };

            // Update the model's constant buffer with the transformation details previously calculated
            model.const_handle().update(
                &mut renderer,
                voxel::ModelConsts {
                    model_mat: *model_mat.as_ref(),
                },
            );

            model.render(&mut renderer, &self.voxel_pipeline, &self.global_consts);
        }

        tonemapper::render(&mut renderer, &self.tonemapper_pipeline, &self.global_consts);

        // Draw ui
        self.ui
            .borrow_mut()
            .render(&mut renderer, &self.client.clone(), &self.window.get_size());

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
