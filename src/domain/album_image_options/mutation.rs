use async_graphql::*;
use async_graphql_relay::RelayNodeID;
use uuid::Uuid;

use crate::{
    db::ModelManager,
    domain::{
        album::Album,
        image::{Image, ImageDao},
    },
    graphql::{AuthGuard, Error},
};

use super::{
    db_model::{
        AlbumImage as DbAlbumImage, AlbumImageDao, CreateAlbumImage as DbCreateAlbumImage,
        UpdateAlbumImage as DbUpdateAlbumImage,
    },
    graphql_model::UpdateAlbumImage,
    AlbumImage,
};

#[derive(Default)]
pub struct AlbumImageMutation;

#[Object]
impl AlbumImageMutation {
    #[graphql(guard = "AuthGuard")]
    async fn update_album_image_options(
        &self,
        ctx: &Context<'_>,
        id: RelayNodeID<AlbumImage>,
        input: UpdateAlbumImage,
    ) -> Result<AlbumImage> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let album_image = AlbumImageDao::update(mm, &id.to_uuid(), &input.into())
            .map(|album_image: DbAlbumImage| -> AlbumImage { album_image.into() })
            .map_err(|e| -> Error { e.into() })?;

        Ok(album_image)
    }

    #[graphql(guard = "AuthGuard")]
    async fn update_album_images_order_index(
        &self,
        ctx: &Context<'_>,
        album_images: Vec<RelayNodeID<AlbumImage>>,
    ) -> Result<Vec<AlbumImage>> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let res = album_images
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                (
                    id.to_uuid(),
                    DbUpdateAlbumImage {
                        album_id: None,
                        order_index: Some(i as i32),
                        highlighted: None,
                        is_primary_album: None,
                    },
                )
            })
            .collect::<Vec<(Uuid, DbUpdateAlbumImage)>>();

        let album_images = AlbumImageDao::update_many(mm, res)?
            .into_iter()
            .map(|album_image| album_image.into())
            .collect();

        Ok(album_images)
    }

    #[graphql(guard = "AuthGuard")]
    async fn move_images_to_album(
        &self,
        ctx: &Context<'_>,
        images: Vec<RelayNodeID<AlbumImage>>,
        album_id: RelayNodeID<Album>,
    ) -> Result<Vec<AlbumImage>> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let mut album_images = Vec::new();
        for image in images {
            let album_image = AlbumImageDao::update(
                mm,
                &image.to_uuid(),
                &DbUpdateAlbumImage {
                    album_id: Some(album_id.to_uuid()),
                    order_index: None,
                    highlighted: None,
                    is_primary_album: None,
                },
            )
            .map(|album_image: DbAlbumImage| -> AlbumImage { album_image.into() })
            .map_err(|e| -> Error { e.into() })?;
            album_images.push(album_image);
        }

        Ok(album_images)
    }

    #[graphql(guard = "AuthGuard")]
    async fn add_images_to_album(
        &self,
        ctx: &Context<'_>,
        album_id: RelayNodeID<Album>,
        images_id: Vec<RelayNodeID<Image>>,
    ) -> Result<Vec<AlbumImage>> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let images_count =
            ImageDao::get_by_album_id(mm, &album_id.to_uuid()).map(|images| images.len())?;

        let mut album_images = Vec::new();
        for image_id in images_id.iter().enumerate() {
            let album_image = DbCreateAlbumImage {
                id: Uuid::new_v4(),
                album_id: album_id.to_uuid(),
                image_id: image_id.1.to_uuid(),
                order_index: images_count as i32 + image_id.0 as i32,
                highlighted: false,
                is_primary_album: false,
            };
            let res = AlbumImageDao::create(mm, &album_image)
                .map(|album_image: DbAlbumImage| -> AlbumImage { album_image.into() })
                .map_err(|e| -> Error { e.into() })?;
            album_images.push(res);
        }

        Ok(album_images)
    }
}
