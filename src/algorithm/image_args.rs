use super::{Args, IntoArgs};
use ::core::{iter, ops::Deref};
use ::image::{
    buffer, flat, DynamicImage, GenericImageView as _, ImageBuffer, Pixel, Primitive, Rgba,
};

fn is_rgba_not_transparent<S: Primitive>(p: Rgba<S>) -> bool {
    p[3] != S::DEFAULT_MIN_VALUE
}

fn is_pixel_not_transparent<P: Pixel>(p: &P) -> bool {
    is_rgba_not_transparent(p.to_rgba())
}

impl<'a, P, C> IntoArgs for &'a ImageBuffer<P, C>
where
    P: Pixel,
    C: Deref<Target = [P::Subpixel]>,
{
    type Pixels = iter::Map<buffer::Pixels<'a, P>, fn(&P) -> bool>;

    fn into_args(self) -> Args<Self::Pixels> {
        let (width, height) = self.dimensions();
        Args::new(
            width as _,
            height as _,
            self.pixels().map(is_pixel_not_transparent),
        )
    }
}

impl<'a> IntoArgs for &'a DynamicImage {
    type Pixels = iter::Map<image::Pixels<'a, DynamicImage>, fn((u32, u32, Rgba<u8>)) -> bool>;

    fn into_args(self) -> Args<Self::Pixels> {
        let (width, height) = self.dimensions();
        Args::new(
            width as _,
            height as _,
            self.pixels().map(|(_, _, p)| is_rgba_not_transparent(p)),
        )
    }
}

impl<'a, Buffer, P> IntoArgs for &'a flat::View<Buffer, P>
where
    Buffer: AsRef<[P::Subpixel]>,
    P: Pixel,
{
    type Pixels = iter::Map<image::Pixels<'a, flat::View<Buffer, P>>, fn((u32, u32, P)) -> bool>;

    fn into_args(self) -> Args<Self::Pixels> {
        let (width, height) = self.dimensions();
        Args::new(
            width as _,
            height as _,
            self.pixels().map(|(_, _, p)| is_pixel_not_transparent(&p)),
        )
    }
}

impl<'a, Buffer, P> IntoArgs for &'a flat::ViewMut<Buffer, P>
where
    Buffer: AsMut<[P::Subpixel]> + AsRef<[P::Subpixel]>,
    P: Pixel,
{
    type Pixels = iter::Map<image::Pixels<'a, flat::ViewMut<Buffer, P>>, fn((u32, u32, P)) -> bool>;

    fn into_args(self) -> Args<Self::Pixels> {
        let (width, height) = self.dimensions();
        Args::new(
            width as _,
            height as _,
            self.pixels().map(|(_, _, p)| is_pixel_not_transparent(&p)),
        )
    }
}
