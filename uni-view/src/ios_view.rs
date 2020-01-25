use libc::c_void;
use std::marker::{Send, Sync};

extern crate objc;
use self::objc::{
    rc::StrongPtr,
    runtime::{Class, Object},
    *,
};
extern crate core_graphics;
use self::core_graphics::{base::CGFloat, geometry::CGRect};

#[repr(C)]
pub struct AppViewObj {
    pub view: *mut Object,
    pub metal_layer: *mut c_void,
    pub maximum_frames: i32,
    pub callback_to_swift: extern "C" fn(arg: i32),
}

pub struct AppView {
    pub view: *mut Object,
    pub scale_factor: f32,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub maximum_frames: i32,
    pub callback_to_swift: extern "C" fn(arg: i32),
}

impl AppView {
    pub fn new(obj: AppViewObj) -> Self {
        let scale_factor = get_scale_factor(obj.view);
        let s: CGRect = unsafe { msg_send![obj.view, frame] };
        let physical = crate::ViewSize {
            width: (s.size.width as f32 * scale_factor) as u32,
            height: (s.size.height as f32 * scale_factor) as u32,
        };
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: physical.width,
            height: physical.height,
            present_mode: wgpu::PresentMode::Vsync,
        };
        let (device, queue) = request_device();
        let surface = wgpu::Surface::create_surface_from_core_animation_layer(obj.metal_layer);
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        AppView {
            view: obj.view,
            scale_factor,
            device,
            queue,
            surface,
            sc_desc,
            swap_chain,
            callback_to_swift: obj.callback_to_swift,
            maximum_frames: obj.maximum_frames,
        }
    }
}

impl crate::GPUContext for AppView {
    fn update_swap_chain(&mut self) {
        let size = self.get_view_size();
        println!("view_size: {:?}", size);
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn get_view_size(&self) -> crate::ViewSize {
        let s: CGRect = unsafe { msg_send![self.view, frame] };
        crate::ViewSize {
            width: (s.size.width as f32 * self.scale_factor) as u32,
            height: (s.size.height as f32 * self.scale_factor) as u32,
        }
    }

    fn normalize_touch_point(&self, touch_point_x: f32, touch_point_y: f32) -> (f32, f32) {
        let size = self.get_view_size();
        (touch_point_x * self.scale_factor / size.width as f32, touch_point_y * self.scale_factor / size.height as f32)
    }
}

fn get_scale_factor(obj: *mut Object) -> f32 {
    let s: CGFloat = unsafe { msg_send![obj, contentScaleFactor] };
    s as f32
}

fn request_device() -> (wgpu::Device, wgpu::Queue) {
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::LowPower },
        wgpu::BackendBit::PRIMARY,
    )
    .unwrap();
    adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions { anisotropic_filtering: false },
        limits: wgpu::Limits::default(),
    })
}
