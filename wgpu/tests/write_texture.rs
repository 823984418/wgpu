//! Tests for texture copy

use crate::common::{initialize_test, TestParameters};

use std::num::NonZeroU32;
use wasm_bindgen_test::*;

#[test]
#[wasm_bindgen_test]
fn write_texture_subset() {
    let size = 256;
    let parameters = TestParameters::default().backend_failure(wgpu::Backends::DX12);
    initialize_test(parameters, |ctx| {
        let tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            dimension: wgpu::TextureDimension::D2,
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            mip_level_count: 1,
            sample_count: 1,
        });
        let data = vec![1u8; size as usize * 2];
        // Write the first two rows
        ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(size),
                rows_per_image: std::num::NonZeroU32::new(size),
            },
            wgpu::Extent3d {
                width: size,
                height: 2,
                depth_or_array_layers: 1,
            },
        );

        ctx.queue.submit(None);

        let read_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (size * size) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &read_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(size),
                    rows_per_image: NonZeroU32::new(size),
                },
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );

        ctx.queue.submit(Some(encoder.finish()));

        let slice = read_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| ());
        ctx.device.poll(wgpu::Maintain::Wait);
        let data: Vec<u8> = slice.get_mapped_range().to_vec();

        for byte in &data[..(size as usize * 2)] {
            assert_eq!(*byte, 1);
        }
        for byte in &data[(size as usize * 2)..] {
            assert_eq!(*byte, 0);
        }
    });
}
