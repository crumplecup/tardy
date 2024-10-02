use crate::{Event, Map};
use std::sync::Arc;
use winit::{event_loop, window};

/// The `lens` module provides the [`Lens`] struct, which holds an application view and methods for
/// interacting with the view.
///
/// # Representing window state with `Lens`
///
/// The purpose of the `Lens` struct is to keep track of application state, the analogue of the
/// `WindowState` struct in the `windows` example of the [`winit`] crate.  Since the data in this
/// struct provides a view of the application to the user, the function is similar to that of
/// lenses in a pair of glasses.  The last time I tried this, I ended up with an `EguiState` and a
/// `GalileoState` and a `WindowState`, and it was starting to feel political, so I went for
/// whimsy.
///
/// This struct ends up as a catch-all holding data intended for display, interactivity flags, and
/// anything else that might come in handy. But for now, it just has a handle to the window, and an
/// optimistic `refresh` flag that isn't wired up to anything yet. As a beginner with
/// [egui]("https://docs.rs/egui/latest/egui/"), I
/// frequently insert these kind of control flags into a struct because the framework renders the
/// view anew every frame.  These flags indicate the need to perform an expensive operation, like
/// loading spatial data to a map, and should only happen once, so I will add a boolean field to
/// the struct to track this granular detail of the application space.
///
/// Eventually I want to be able to share a window between the well-tested `egui` library and the
/// relatively immature [galileo](https://docs.rs/galileo/latest/galileo/) library, but for now we
/// are just stubbing this out for future use by wrapping it in an [`Arc`].
#[derive(derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_", into, borrow_self)]
pub struct Lens {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub adapter: accesskit_winit::Adapter,
    proxy: event_loop::EventLoopProxy<Event>,
    refresh: bool,
    window: Arc<window::Window>,
    pub map: Map,
}

impl Lens {
    /// The `new` method creates an instance of `Lens` from an [`Arc<window::Window>`].
    pub async fn new(
        adapter: accesskit_winit::Adapter,
        proxy: event_loop::EventLoopProxy<Event>,
        window: Arc<window::Window>,
    ) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let wgpu_adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = wgpu_adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits {
                            // NOTE(alexkirsz) These are the limits on my GPU w/ WebGPU,
                            // but your mileage may vary.
                            max_texture_dimension_2d: 16384,
                            ..wgpu::Limits::downlevel_webgl2_defaults()
                        }
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&wgpu_adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let surface = Arc::new(surface);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let map = Map::new(
            Arc::clone(&window),
            Arc::clone(&device),
            Arc::clone(&surface),
            Arc::clone(&queue),
            config.clone(),
        );
        Self {
            surface,
            device,
            queue,
            config,
            size,
            adapter,
            proxy,
            refresh: false,
            window,
            map,
        }
    }

    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    pub fn about_to_wait(&mut self) {
        self.map.about_to_wait();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.map.resize(new_size);
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let texture = self.surface.get_current_texture()?;

        let texture_view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let frame = crate::map::Frame {
                device: &self.device,
                queue: &self.queue,
                encoder: &mut encoder,
                window: &self.window,
                texture_view: &texture_view,
                size: self.size,
            };

            self.map.render(&frame);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        texture.present();

        Ok(())
    }
}
