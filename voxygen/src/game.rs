// Ui
use ui::Ui;

// Standard
use std::{
    cell::RefCell,
    f32::consts::PI,
    net::ToSocketAddrs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
//use std::f32::{sin, cos};

// Library
use coord::prelude::*;
use dot_vox;
use fnv::FnvBuildHasher;
use glutin::ElementState;
use indexmap::IndexMap;
use nalgebra::{Rotation3, Translation3, Vector2, Vector3};
use parking_lot::Mutex;

type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;

// Project
use client::{self, Client, ClientMode, CHUNK_SIZE};
use region::{
    chunk::{Chunk, ChunkContainer, ChunkConverter},
    Container, PersState, VolContainer, VolConverter, VolState,
};

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
    Meshes(FnvIndexMap<voxel::MaterialKind, voxel::Mesh>),
    Model {
        model: voxel::Model,
        model_consts: ConstHandle<voxel::ModelConsts>,
    },
}

pub struct Payloads {}
impl client::Payloads for Payloads {
    type Chunk = ChunkPayload;
    type Entity = ConstHandle<voxel::ModelConsts>;
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
    volume_pipeline: voxel::VolumePipeline,
    tonemapper_pipeline: Pipeline<tonemapper::pipeline::Init<'static>>,

    skybox_model: skybox::Model,
    player_model: voxel::Model,
    other_player_model: voxel::Model,
}

fn gen_payload(key: Vec3<i64>, con: &Container<ChunkContainer, <Payloads as client::Payloads>::Chunk>) {
    con.set_access();
    if con.vols().get(PersState::Raw).is_none() {
        //only get mutable lock if no Raw exists
        ChunkConverter::convert(&key, &mut con.vols_mut(), PersState::Raw);
    }
    if let Some(raw) = con.vols().get(PersState::Raw) {
        let chunk: &Chunk = raw.as_any().downcast_ref::<Chunk>().expect("Should be Chunk");
        *con.payload_mut() = Some(ChunkPayload::Meshes(voxel::Mesh::from(chunk)));
    } else {
        let vols = con.vols();
        panic!(
            "Raw chunk {} does not exist, rle: {}, file: {}",
            key,
            vols.get(PersState::Rle).is_some(),
            vols.get(PersState::File).is_some()
        );
    }
}

impl Game {
    pub fn new<R: ToSocketAddrs>(mode: ClientMode, alias: &str, remote_addr: R, view_distance: i64) -> Game {
        let window = RenderWindow::new();

        let client = Client::new(mode, alias.to_string(), remote_addr, gen_payload, view_distance)
            .expect("Could not create new client");
        client.start();

        // Contruct the UI
        let window_dims = window.get_size();
        let ui = Ui::new(&mut window.renderer_mut(), window_dims, &client);

        // Create pipelines

        let volume_pipeline = voxel::VolumePipeline::new(&mut window.renderer_mut());

        let skybox_pipeline = Pipeline::new(
            window.renderer_mut().factory_mut(),
            skybox::pipeline::new(),
            &Shader::from_file("shaders/skybox/skybox.vert").expect("Could not load skybox vertex shader"),
            &Shader::from_file("shaders/skybox/skybox.frag").expect("Could not load skybox fragment shader"),
        );

        let tonemapper_pipeline = Pipeline::new(
            window.renderer_mut().factory_mut(),
            tonemapper::pipeline::new(),
            &Shader::from_file("shaders/tonemapper/tonemapper.vert").expect("Could not load skybox vertex shader"),
            &Shader::from_file("shaders/tonemapper/tonemapper.frag").expect("Could not load skybox fragment shader"),
        );

        let global_consts = ConstHandle::new(&mut window.renderer_mut());

        let skybox_mesh = skybox::Mesh::new_skybox();
        let skybox_model = skybox::Model::new(&mut window.renderer_mut(), &skybox_mesh);

        info!("trying to load model files");
        let vox = dot_vox::load("assets/cosmetic/creature/friendly/player3.vox")
            .expect("cannot find model 3.vox. Make sure to start voxygen from its folder");
        let voxmodel = voxel::vox_to_figure(vox);

        let player_meshes = voxel::Mesh::from_with_offset(&voxmodel, vec3!(-10.0, -4.0, 0.0));

        let player_model = voxel::Model::new(&mut window.renderer_mut(), &player_meshes);

        let vox = dot_vox::load("assets/cosmetic/creature/friendly/player5.vox")
            .expect("cannot find model 5.vox. Make sure to start voxygen from its folder");
        let voxmodel = voxel::vox_to_figure(vox);

        let other_player_meshes = voxel::Mesh::from(&voxmodel);

        let other_player_model = voxel::Model::new(&mut window.renderer_mut(), &other_player_meshes);

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
            volume_pipeline,
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
                            .rotate_by(Vector2::new(dx as f32 * 0.002, dy as f32 * 0.002));
                    }
                },
                Event::MouseWheel { dy, .. } => {
                    self.camera.lock().zoom_by((-dy / 4.0) as f32);
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
                            self.key_state.lock().up = match i.state {
                                // Default: W (up)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.left, i.scancode) {
                            self.key_state.lock().left = match i.state {
                                // Default: A (left)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.back, i.scancode) {
                            self.key_state.lock().down = match i.state {
                                // Default: S (down)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.right, i.scancode) {
                            self.key_state.lock().right = match i.state {
                                // Default: D (right)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.jump, i.scancode) {
                            self.key_state.lock().jump = match i.state {
                                // Default: Space (fly)
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        } else if keypress_eq(&general.crouch, i.scancode) {
                            // self.key_state.lock().fall = match i.state { // Default: Shift (fall)
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
                        .set_aspect_ratio((w.max(1) as f32) / (h.max(1) as f32));
                },
                _ => {},
            }
        });

        // Calculate movement player movement vector
        let ori = *self.camera.lock().ori();
        let unit_vecs = (
            Vector2::new(ori.x.cos(), -ori.x.sin()),
            Vector2::new(ori.x.sin(), ori.x.cos()),
        );
        let dir_vec = self.key_state.lock().dir_vec();
        let mov_vec = unit_vecs.0 * dir_vec.x + unit_vecs.1 * dir_vec.y;

        // Why do we do this in Voxygen?!
        const LOOKING_VEL_FAC: f32 = 1.0;
        const LOOKING_CTRL_ACC_FAC: f32 = 1.0;
        const MIN_LOOKING: f32 = 0.5;
        const LEANING_FAC: f32 = 0.05;
        if let Some(player_entity) = self.client.player_entity() {
            let mut player_entity = player_entity.write();

            // Apply acceleration
            player_entity.ctrl_acc_mut().x = mov_vec.x;
            player_entity.ctrl_acc_mut().y = mov_vec.y;

            // Apply jumping
            player_entity.ctrl_acc_mut().z = if self.key_state.lock().jump() { 1.0 } else { 0.0 };

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

        self.running.load(Ordering::Relaxed)
    }

    pub fn update_chunks(&self) {
        let mut renderer = self.window.renderer_mut();
        // Find the chunk the camera is in
        let cam_origin = *self.camera.lock().get_pos();
        let cam_chunk = Vec3::<i64>::new(
            (cam_origin.x as i64).div_euc(CHUNK_SIZE[0]),
            (cam_origin.y as i64).div_euc(CHUNK_SIZE[1]),
            (cam_origin.z as i64).div_euc(CHUNK_SIZE[2]),
        );

        for (pos, con) in self.client.chunk_mgr().persistence().hot().iter() {
            // TODO: Fix this View Distance which only take .x into account and describe the algorithm what it should do exactly!
            if (*pos - cam_chunk).snake_length() > (self.client.view_distance() as i64 * 2) / CHUNK_SIZE[0] {
                continue;
            }
            con.set_access();
            {
                let mut trylock = &mut con.payload_try_mut(); //we try to lock it, if it is already written to we just ignore this chunk for a frame
                if let Some(ref mut lock) = trylock {
                    //sometimes persistence does not exist, dont render then
                    if let Some(ref mut payload) = **lock {
                        if let ChunkPayload::Meshes(ref mut mesh) = payload {
                            // Calculate chunk mode matrix
                            let model_mat = &Translation3::<f32>::from_vector(Vector3::<f32>::new(
                                (pos.x * CHUNK_SIZE[0]) as f32,
                                (pos.y * CHUNK_SIZE[1]) as f32,
                                (pos.z * CHUNK_SIZE[2]) as f32,
                            )).to_homogeneous();

                            // Create set new model constants
                            let model_consts = ConstHandle::new(&mut renderer);

                            // Update chunk model constants
                            model_consts.update(
                                &mut renderer,
                                voxel::ModelConsts {
                                    model_mat: *model_mat.as_ref(),
                                },
                            );

                            // Update the chunk payload
                            *payload = ChunkPayload::Model {
                                model: voxel::Model::new(&mut renderer, mesh),
                                model_consts,
                            };
                        }
                    }
                }
            }
        }
    }

    pub fn update_entities(&self) {
        // Take the physics lock to sync client and frontend updates
        let _ = self.client.take_phys_lock();

        // Set camera focus to the player's head
        if let Some(player_entity) = self.client.player_entity() {
            let player_entity = player_entity.read();
            self.camera.lock().set_focus(Vector3::<f32>::from(
                (*player_entity.pos() + vec3!(0.0, 0.0, 1.75)).elements(),
            ));
        }

        let mut renderer = self.window.renderer_mut();

        // Update each entity constbuffer
        for (uid, entity) in self.client.entities().iter() {
            let mut entity = entity.write();

            // Calculate entity model matrix
            let model_mat = &Translation3::from_vector(Vector3::from(entity.pos().elements())).to_homogeneous()
                * Rotation3::new(Vector3::new(0.0, 0.0, PI - entity.look_dir().x)).to_homogeneous()
                * Rotation3::new(Vector3::new(entity.look_dir().y, 0.0, 0.0)).to_homogeneous();

            // Update the model const buffer (its payload)
            // TODO: Put the model into the payload so we can have per-entity models!
            entity
                .payload_mut()
                .get_or_insert_with(|| ConstHandle::new(&mut renderer))
                .update(
                    &mut renderer,
                    voxel::ModelConsts {
                        model_mat: *model_mat.as_ref(),
                    },
                );
        }
    }

    pub fn render_frame(&mut self) {
        // Calculate frame constants
        let camera_mats = self.camera.lock().get_mats();
        let cam_origin = *self.camera.lock().get_pos();
        let play_origin = self
            .client
            .player_entity()
            .map(|p| *p.read().pos())
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

        // Find the chunk the camera is in
        let cam_chunk = Vec3::<i64>::new(
            (cam_origin.x as i64).div_euc(CHUNK_SIZE[0]),
            (cam_origin.y as i64).div_euc(CHUNK_SIZE[1]),
            (cam_origin.z as i64).div_euc(CHUNK_SIZE[2]),
        );

        // Render each chunk
        for (pos, con) in self.client.chunk_mgr().persistence().hot().iter() {
            // TODO: Fix this View Distance which only take .x into account and describe the algorithm what it should do exactly!
            if (*pos - cam_chunk).snake_length() > (self.client.view_distance() as i64 * 2) / CHUNK_SIZE[0] {
                continue;
            }
            // rendering actually does not set the time, but updating does it
            let trylock = &con.payload_try(); //we try to lock it, if it is already written to we just ignore this chunk for a frame
            if let Some(ref lock) = trylock {
                if let Some(ref payload) = **lock {
                    if let ChunkPayload::Model {
                        ref model,
                        ref model_consts,
                    } = payload
                    {
                        self.volume_pipeline
                            .draw_model(&model, model_consts, &self.global_consts);
                    }
                }
            }
        }

        // Render each entity
        for (uid, entity) in self.client.entities().iter() {
            // Choose the correct model for the entity
            let model = match self.client.player().entity_uid {
                Some(uid) if uid == uid => &self.player_model,
                _ => &self.other_player_model,
            };

            if let Some(ref model_consts) = entity.read().payload() {
                self.volume_pipeline
                    .draw_model(&model, model_consts, &self.global_consts);
            }
        }

        // flush voxel pipeline draws
        self.volume_pipeline.flush(&mut renderer);

        tonemapper::render(&mut renderer, &self.tonemapper_pipeline, &self.global_consts);

        // Draw ui
        self.ui
            .borrow_mut()
            .render(&mut renderer, &self.client.clone(), &self.window.get_size());

        self.window.swap_buffers();
        renderer.end_frame();
    }

    pub fn run(&mut self) {
        while self.handle_window_events() {
            self.update_chunks();
            self.update_entities();
            self.render_frame();
        }

        self.client.shutdown();
    }
}
