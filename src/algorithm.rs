#[cfg(feature = "bevy")]
#[cfg_attr(doc, doc(cfg(feature = "bevy")))]
mod bevy_args;
#[cfg(feature = "image")]
#[cfg_attr(doc, doc(cfg(feature = "image")))]
mod image_args;

use crate::core::{self, Cell, HorizontalIndices, VerticalIndices, Vertices};
use ::alloc::vec::Vec;
use ::core::iter;

#[derive(Clone, Debug, Default)]
pub struct Algorithm(Vec<Cell>);

impl Algorithm {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn with_capacity(width: usize, height: usize) -> Self {
        Self(Vec::with_capacity(core::capacity(width, height)))
    }

    pub fn search(
        &mut self,
        args: impl IntoArgs,
    ) -> (Vertices, iter::Chain<HorizontalIndices, VerticalIndices>) {
        let args = args.into_args();
        let old_len = self.0.len();
        let new_len = core::capacity(args.width, args.height);
        self.0.resize(new_len, Cell::EMPTY);
        core::clear(&mut self.0[..old_len.min(new_len)]);
        core::set(
            &mut self.0,
            args.width,
            args.pixels.into_iter().take(args.width * args.height),
        );
        let (vertices, horizontal_indices, vertical_indices) = core::get(&self.0, args.width);
        (vertices, horizontal_indices.chain(vertical_indices))
    }
}

#[derive(Copy, Clone)]
pub struct Args<P> {
    pub width: usize,
    pub height: usize,
    pub pixels: P,
}

impl<P> Args<P> {
    pub const fn new(width: usize, height: usize, pixels: P) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }
}

pub trait IntoArgs {
    type Pixels: IntoIterator<Item = bool>;

    fn into_args(self) -> Args<Self::Pixels>;
}

impl<P: IntoIterator<Item = bool>> IntoArgs for Args<P> {
    type Pixels = P;

    #[inline]
    fn into_args(self) -> Args<Self::Pixels> {
        self
    }
}
