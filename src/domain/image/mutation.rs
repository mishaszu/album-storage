use async_graphql::*;
use async_graphql_relay::RelayNodeID;
use imagesize::blob_size;
use reqwest::Client;
use uuid::Uuid;

use crate::{
    config::config,
    db::ModelManager,
    domain::album::Album,
    graphql::{AuthGuard, Error},
    services::lust::Lust,
    utils::{delete_file, read_file},
};

use super::{
    db_model::{CreateImage as DbCreateImage, Image as DbImage, ImageDao},
    graphql_model::{CreateImage, UpdateImage},
    Image,
};

#[derive(Default)]
pub struct ImageMutation;

#[Object]
impl ImageMutation {
    #[graphql(guard = "AuthGuard")]
    async fn create_image(&self, ctx: &Context<'_>, input: CreateImage) -> Result<Image> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };
        let image = ImageDao::create(mm, input.into())
            .map(|image: DbImage| -> Image { image.into() })
            .map_err(|e| -> Error { e.into() })?;
        Ok(image)
    }

    #[graphql(guard = "AuthGuard")]
    async fn update_image(
        &self,
        ctx: &Context<'_>,
        id: RelayNodeID<Image>,
        input: UpdateImage,
    ) -> Result<Image> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };
        let image = ImageDao::update(mm, &id.to_uuid(), input.into())
            .map(|image: DbImage| -> Image { image.into() })
            .map_err(|e| -> Error { e.into() })?;
        Ok(image)
    }

    #[graphql(guard = "AuthGuard")]
    async fn delete_image(&self, ctx: &Context<'_>, id: RelayNodeID<Image>) -> Result<Image> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let client = ctx.data_opt::<Client>();
        let client = match client {
            Some(client) => client,
            None => return Err(Error::ClientNotInContext.into()),
        };

        let image = ImageDao::get_by_id(mm, &id.to_uuid()).map_err(|e| -> Error { e.into() })?;

        Lust::delete_file(client, &config().LUST_BUCKET, &image.path)
            .await
            .map_err(|_| -> Error { Error::FailedToDeleteFile })?;

        let image = ImageDao::delete(mm, &id.to_uuid()).map_err(|e| -> Error { e.into() })?;

        Ok(image.into())
    }

    #[graphql(guard = "AuthGuard")]
    async fn upload_image(
        &self,
        ctx: &Context<'_>,
        album_id: RelayNodeID<Album>,
        is_primary_album: bool,
        album_path: String,
        image_path: String,
    ) -> Result<Image> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let client = match ctx.data_opt::<Client>() {
            Some(client) => client,
            None => return Err(Error::ClientNotInContext.into()),
        };

        let image_name = format!("{}/{}", album_path, image_path.clone());
        let file = read_file(&image_name)
            .await
            .map_err(|_| Error::FailedToReadFile)?;

        let image_dimensions = blob_size(&file).map_err(|_| Error::BadImage)?;

        let response = Lust::post_file(client, &config().LUST_BUCKET, file)
            .await
            .map_err(|e| Error::GraphQlError(e.into()))?;

        let is_image_uploaded =
            ImageDao::is_uploaded(mm, &image_name).map_err(|e| -> Error { e.into() })?;

        if is_image_uploaded {
            return Err(Error::EntityExists.into());
        }

        let image_res = ImageDao::create_with_album(
            mm,
            &album_id.to_uuid(),
            DbCreateImage {
                id: Uuid::new_v4(),
                title: image_path.clone(),
                description: None,
                original_full_title: image_name.clone(),
                path: response.image_id,
                width: Some(image_dimensions.width as i32),
                height: Some(image_dimensions.height as i32),
                is_uploaded: true,
            },
            None,
            is_primary_album,
        )
        .map(|image: DbImage| -> Image { image.into() })
        .map_err(|e| -> Error { e.into() })?;

        delete_file(&image_name)
            .await
            .map_err(|_| Error::FailedToDeleteFile)?;

        Ok(image_res)
    }
}
