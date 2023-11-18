use super::{Args, IntoArgs};
use ::bevy::render::{render_resource::TextureFormat, texture::Image};
use ::core::{convert, iter, slice};

impl<'a> IntoArgs for &'a Image {
    type Pixels = iter::FilterMap<
        iter::FlatMap<
            slice::ChunksExact<'a, u8>,
            [Option<bool>; 2],
            fn(&[u8]) -> [Option<bool>; 2],
        >,
        fn(Option<bool>) -> Option<bool>,
    >;

    fn into_args(self) -> Args<Self::Pixels> {
        Args::new(
            self.width() as _,
            self.height() as _,
            match self.texture_descriptor.format {
                TextureFormat::Rgba8Unorm
                | TextureFormat::Rgba8UnormSrgb
                | TextureFormat::Rgba8Snorm
                | TextureFormat::Rgba8Uint
                | TextureFormat::Rgba8Sint
                | TextureFormat::Bgra8Unorm
                | TextureFormat::Bgra8UnormSrgb => self
                    .data
                    .chunks_exact(4)
                    .flat_map::<_, fn(&[u8]) -> [Option<bool>; 2]>(|pixel| {
                        [Some(pixel[3] != 0), None]
                    })
                    .filter_map(convert::identity),

                TextureFormat::Rgb10a2Unorm => self
                    .data
                    .chunks_exact(3)
                    .flat_map::<_, fn(&[u8]) -> [Option<bool>; 2]>(|pixel| {
                        [
                            Some(pixel[2] & 0b00001100 != 0),
                            Some(pixel[3] & 0b11000000 != 0),
                        ]
                    })
                    .filter_map(convert::identity),
                TextureFormat::Rgba16Uint
                | TextureFormat::Rgba16Sint
                | TextureFormat::Rgba16Unorm
                | TextureFormat::Rgba16Snorm
                | TextureFormat::Rgba16Float => self
                    .data
                    .chunks_exact(8)
                    .flat_map::<_, fn(&[u8]) -> [Option<bool>; 2]>(|pixel| {
                        [Some(&pixel[6..] != &[0; 2]), None]
                    })
                    .filter_map(convert::identity),
                TextureFormat::Rgba32Uint
                | TextureFormat::Rgba32Sint
                | TextureFormat::Rgba32Float => self
                    .data
                    .chunks_exact(16)
                    .flat_map::<_, fn(&[u8]) -> [Option<bool>; 2]>(|pixel| {
                        [Some(&pixel[12..] != &[0; 4]), None]
                    })
                    .filter_map(convert::identity),
                _ => unimplemented!(
                    "only uncompressed texture formats with alpha channels are currently supported for bevy"
                ),
            },
        )
    }
}
