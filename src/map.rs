use galileo::galileo_types::latlon;
use std::sync::{Arc, RwLock};

#[derive(derive_getters::Getters)]
pub struct Map {
    delegate: galileo::winit::WinitInputHandler,
    renderer: Arc<RwLock<galileo::render::WgpuRenderer>>,
    content: Arc<RwLock<galileo::Map>>,
}

impl Map {
    pub fn new(
        window: Arc<winit::window::Window>,
        device: Arc<wgpu::Device>,
        surface: Arc<wgpu::Surface<'static>>,
        queue: Arc<wgpu::Queue>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let renderer = galileo::render::WgpuRenderer::new_with_device_and_surface(
            device, surface, queue, config,
        );
        let renderer = Arc::new(RwLock::new(renderer));
        let messenger = galileo::winit::WinitMessenger::new(window);
        let view = galileo::MapView::new(
            &latlon!(42.4434, -123.3252),
            galileo::tile_scheme::TileSchema::web(18)
                .lod_resolution(13)
                .unwrap(),
        );

        let tile_source = |index: &galileo::tile_scheme::TileIndex| {
            format!(
                "https://tile.openstreetmap.org/{}/{}/{}.png",
                index.z, index.x, index.y
            )
        };

        let tile_layer = Box::new(galileo::MapBuilder::create_raster_tile_layer(
            tile_source,
            galileo::tile_scheme::TileSchema::web(18),
        ));

        let content = galileo::Map::new(view, vec![tile_layer], Some(messenger));
        let content = Arc::new(RwLock::new(content));

        Self {
            delegate: Default::default(),
            renderer,
            content,
        }
    }

    pub fn delegate_mut(&mut self) -> &mut galileo::winit::WinitInputHandler {
        &mut self.delegate
    }

    pub fn about_to_wait(&self) {
        self.content.write().unwrap().animate();
    }

    pub fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        self.renderer
            .write()
            .expect("poisoned lock")
            .resize(galileo_types::cartesian::Size::new(size.width, size.height));
        self.content
            .write()
            .expect("poisoned lock")
            .set_size(galileo_types::cartesian::Size::new(
                size.width as f64,
                size.height as f64,
            ));
    }

    pub fn render(&self, frame: &Frame<'_>) {
        let content = self.content.read().unwrap();
        content.load_layers();

        self.renderer
            .write()
            .expect("poisoned lock")
            .render_to_texture_view(&content, frame.texture_view);
    }
}

pub struct Frame<'frame> {
    pub device: &'frame wgpu::Device,
    pub queue: &'frame wgpu::Queue,
    pub encoder: &'frame mut wgpu::CommandEncoder,
    pub window: &'frame winit::window::Window,
    pub texture_view: &'frame wgpu::TextureView,
    pub size: winit::dpi::PhysicalSize<u32>,
}
