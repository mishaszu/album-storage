use async_graphql::{ComplexObject, Context, InputObject, SimpleObject};
use async_graphql_relay::{RelayNode, RelayNodeID, RelayNodeObject};
use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    db::ModelManager,
    domain::album::{Album, DbAlbum},
    domain::album_image_options::{AlbumImage, AlbumImageDao, DbAlbumImage},
    graphql::{node::Node, Error, Identifiable},
};

use super::{DbCreateImage, DbImage, DbUpdateImage, ImageDao};

#[derive(SimpleObject, RelayNodeObject, Debug, Clone)]
#[graphql(complex)]
#[relay(node_suffix = "im")]
pub struct Image {
    pub id: RelayNodeID<Self>,
    pub title: String,
    pub original_full_title: String,
    pub description: Option<String>,
    pub path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub is_uploaded: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[ComplexObject]
impl Image {
    async fn albums_options(&self, ctx: &Context<'_>) -> Result<Vec<AlbumImage>, Error> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext),
        };
        let album_images = AlbumImageDao::get_by_image_id(mm, &self.id.to_uuid())
            .map(|album_images: Vec<DbAlbumImage>| -> Vec<AlbumImage> {
                album_images
                    .into_iter()
                    .map(|album_image: DbAlbumImage| -> AlbumImage { album_image.into() })
                    .collect()
            })
            .map_err(|e| -> Error { e.into() })?;
        Ok(album_images)
    }

    async fn albums(&self, ctx: &Context<'_>) -> Result<Vec<Album>, Error> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext),
        };
        let albums = ImageDao::get_albums(mm, &self.id.to_uuid())
            .map(|albums: Vec<DbAlbum>| -> Vec<Album> {
                albums
                    .into_iter()
                    .map(|album: DbAlbum| -> Album { album.into() })
                    .collect()
            })
            .map_err(|e| -> Error { e.into() })?;
        Ok(albums)
    }
}

impl Identifiable for Image {
    fn get_id(&self) -> Uuid {
        self.id.to_uuid()
    }
}

#[async_trait]
impl RelayNode for Image {
    type TNode = Node;

    async fn get(
        ctx: async_graphql_relay::RelayContext,
        id: RelayNodeID<Self>,
    ) -> async_graphql::Result<Option<Self::TNode>> {
        let mm = ctx.get::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };
        let album = ImageDao::get_by_id(mm, &id.to_uuid())
            .map(|image: DbImage| -> Image { image.into() })
            .map_err(|e| -> Error { e.into() })?;
        Ok(Some(album.into()))
    }
}

impl From<DbImage> for Image {
    fn from(image: DbImage) -> Self {
        Self {
            id: RelayNodeID::new(image.id),
            title: image.title,
            original_full_title: image.original_full_title,
            description: image.description,
            path: image.path,
            width: image.width,
            height: image.height,
            is_uploaded: image.is_uploaded,
            created_at: image.created_at,
            updated_at: image.updated_at,
        }
    }
}

#[derive(InputObject)]
pub struct CreateImage {
    pub title: String,
    pub description: Option<String>,
    pub original_title: String,
    pub path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size_bytes: i32,
    pub photoed_at: Option<chrono::NaiveDateTime>,
    pub is_ready: bool,
    pub is_uploaded: bool,
}

impl From<CreateImage> for DbCreateImage {
    fn from(val: CreateImage) -> Self {
        DbCreateImage {
            id: Uuid::new_v4(),
            title: val.title,
            description: val.description,
            original_full_title: val.original_title,
            path: val.path,
            width: val.width,
            height: val.height,
            is_uploaded: val.is_uploaded,
        }
    }
}

#[derive(InputObject)]
pub struct UpdateImage {
    pub title: Option<String>,
    pub description: Option<String>,
    pub original_full_title: Option<String>,
    pub path: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub is_uploaded: Option<bool>,
}

impl From<UpdateImage> for DbUpdateImage {
    fn from(val: UpdateImage) -> Self {
        DbUpdateImage {
            title: val.title,
            description: val.description,
            original_full_title: val.original_full_title,
            path: val.path,
            width: val.width,
            height: val.height,
            is_uploaded: val.is_uploaded,
        }
    }
}
