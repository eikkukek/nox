use std::sync::{Arc, RwLock};

use nox_mem::{
    slot_map::{GlobalSlotMap, SlotIndex},
};

use super::{
    Error,
    image::{
        ImageBuilder,
        Image,
        ImageRangeInfo,
        ImageSubresourceRange,
        ImageSource,
        ImageSourceMut,
    },
    memory_binder::MemoryBinder,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageID(pub(crate) SlotIndex<ImageResource>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageSubresourceID(pub(crate) SlotIndex<ImageResource>, pub(crate) SlotIndex<ImageSubresourceRange>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageSourceID {
    ImageID(ImageID),
    SubresourceID(ImageSubresourceID),
}

impl From<ImageID> for ImageSourceID {

    fn from(value: ImageID) -> Self {
        Self::ImageID(value)
    }
}

impl From<ImageSubresourceID> for ImageSourceID {

    fn from(value: ImageSubresourceID) -> Self {
        Self::SubresourceID(value)
    }
}

pub(crate) struct ImageResource {
    subresources: GlobalSlotMap<ImageSubresourceRange>,
    image: Image,
}

pub struct GlobalResources {
    device: Arc<ash::Device>,
    images: GlobalSlotMap<ImageResource>,
}

impl GlobalResources {

    pub(crate) fn new(device: Arc<ash::Device>) -> Self {
        Self {
            device,
            images: GlobalSlotMap::new(),
        }
    }

    #[inline(always)]
    pub fn create_image<F, Binder: MemoryBinder>(
        &mut self,
        mut f: F,
        binder: &mut Binder,
    ) -> Result<ImageID, Error>
        where
            F: FnMut(&mut ImageBuilder)
    {
        let mut builder = ImageBuilder::new(self.device.clone());
        f(&mut builder);
        let mut image = builder.build()?;
        unsafe {
            image.set_memory(Box::new(binder.bind_image_memory(image.handle())?));
        }
        Ok(ImageID(
            self.images.insert(ImageResource {
                image,
                subresources: Default::default(),
            })
        ))
    }

    #[inline(always)]
    pub fn create_image_subresource(
        s: Arc<RwLock<Self>>,
        id: ImageID,
        range_info: ImageRangeInfo
    ) -> Result<ImageSubresourceID, Error>
    {
        let subresource = ImageSubresourceRange::new(range_info, id, s.clone())?;
        let mut s = s.write().unwrap();
        Ok(ImageSubresourceID(id.0, s.images
            .get_mut(id.0)
            .subresources.insert(subresource)
        ))
    }

    #[inline(always)]
    pub fn destroy_image(&mut self, id: ImageID) {
        self.images.remove(id.0);
    }

    #[inline(always)]
    pub fn destroy_image_subresource(&mut self, id: ImageSubresourceID) {
        self.images.get_mut(id.0).subresources.remove(id.1);
    }

    #[inline(always)]
    pub fn destroy_image_source(&mut self, id: ImageSourceID) {
        match id {
            ImageSourceID::ImageID(id) => {
                self.destroy_image(id);
            },
            ImageSourceID::SubresourceID(id) => {
                self.destroy_image_subresource(id);
            },
        }
    }

    #[inline(always)]
    pub fn is_valid_image_id(&self, id: ImageSourceID) -> bool {
        match id {
            ImageSourceID::ImageID(id) => {
                self.images.contains(id.0)
            },
            ImageSourceID::SubresourceID(id) => {
                self.images
                    .try_get(id.0)
                    .map(|v| v.subresources.contains(id.1))
                    .unwrap_or(false)
            },
        }
    }

    #[inline(always)]
    pub(crate) fn get_image(
        &self,
        id: ImageID,
    ) -> &Image
    {
        &self.images.get(id.0).image
    }

    #[inline(always)]
    pub(crate) fn _get_mut_image(
        &mut self,
        id: ImageID,
    ) -> &mut Image
    {
        &mut self.images.get_mut(id.0).image
    }

    #[inline(always)]
    pub(crate) fn _get_image_subresource(
        &self,
        id: ImageSubresourceID,
    ) -> &ImageSubresourceRange
    {
        self.images
            .get(id.0)
            .subresources
            .get(id.1)
    }

    #[inline(always)]
    pub(crate) fn get_mut_image_subresource(
        &mut self,
        id: ImageSubresourceID,
    ) -> &mut ImageSubresourceRange
    {
        self.images
            .get_mut(id.0)
            .subresources
            .get_mut(id.1)
    }

    #[inline(always)]
    pub(crate) fn get_image_source(
        &self,
        id: ImageSourceID,
    ) -> ImageSource<'_>
    {
        match id {
            ImageSourceID::ImageID(id) => {
                ImageSource::Image(
                    &self.images.get(id.0).image
                )
            },
            ImageSourceID::SubresourceID(id) => {
                ImageSource::Subresource(
                    self.images
                        .get(id.0)
                        .subresources
                        .get(id.1)
                )
            }
        }
    }

    #[inline(always)]
    pub(crate) fn get_mut_image_source(
        &mut self,
        id: ImageSourceID,
    ) -> ImageSourceMut<'_>
    {
        match id {
            ImageSourceID::ImageID(id) => {
                ImageSourceMut::Image(
                    &mut self.images.get_mut(id.0).image
                )
            },
            ImageSourceID::SubresourceID(id) => {
                ImageSourceMut::Subresource(self.images
                    .get_mut(id.0)
                    .subresources
                    .get_mut(id.1)
                )
            }
        }
    }
}
