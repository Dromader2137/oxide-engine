use std::sync::Arc;

use log::debug;
use vulkano::{device::{physical::{PhysicalDevice, PhysicalDeviceType}, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags}, instance::{Instance, InstanceCreateInfo}, swapchain::Surface, VulkanLibrary};

use crate::rendering::Window;

pub struct VulkanContext {
    pub library: Arc<VulkanLibrary>,
    pub instance: Arc<Instance>,
    pub physical_device: Arc<PhysicalDevice>,
    pub render_surface: Arc<Surface>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub transfer_queue: Arc<Queue>,
}

fn select_physical_device(
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device_extensions: &DeviceExtensions,
    features: &Features,
) -> (Arc<PhysicalDevice>, u32, Option<u32>) {
    instance
        .enumerate_physical_devices()
        .expect("failed to enumerate physical devices")
        .filter(|p| p.supported_extensions().contains(device_extensions))
        .filter(|p| p.supported_features().contains(features))
        .filter_map(|p| {
            let gq = p
                .queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface.clone())
                            .unwrap_or(false)
                })
                .map(|q| q as u32);
            let tq = p
                .queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::TRANSFER)
                        && i as u32 != gq.expect("No graphics queue")
                })
                .map(|q| q as u32);

            debug!("Selected queues main:{:?}, transfer:{:?}", gq, tq);

            gq.map(|gq| (p, gq, tq))
        })
        .min_by_key(|(p, _, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("no device available")
}

impl VulkanContext {
    pub fn new(window: &Window) -> VulkanContext {
        let features = Features {
            shader_draw_parameters: true,
            sampler_anisotropy: true,
            fill_mode_non_solid: true,
            ..Features::empty()
        };
        let extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let library = VulkanLibrary::new().expect("Vulkan library not found");
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: Surface::required_extensions(&window.window_handle),
                ..Default::default()
            },
        )
        .unwrap();

        let surface = Surface::from_window(instance.clone(), window.window_handle.clone()).unwrap();
        let (physical_device, queue_family_index, transfer_family_index) =
            select_physical_device(instance.clone(), surface.clone(), &extensions, &features);

        debug!("Vulkan version: {}", instance.api_version());

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: {
                    if transfer_family_index.is_some() {
                        vec![
                            QueueCreateInfo {
                                queue_family_index,
                                ..Default::default()
                            },
                            QueueCreateInfo {
                                queue_family_index: transfer_family_index.unwrap(),
                                ..Default::default()
                            },
                        ]
                    } else {
                        vec![QueueCreateInfo {
                            queue_family_index,
                            ..Default::default()
                        }]
                    }
                },
                enabled_extensions: extensions,
                enabled_features: features,
                ..Default::default()
            },
        )
        .unwrap();
        let queue = queues.next().unwrap();
        let transfer_queue = queues.next().unwrap_or(queue.clone());

        VulkanContext {
            library, 
            instance,
            physical_device,
            device,
            render_surface: surface,
            queue,
            transfer_queue
        }

    }
}
