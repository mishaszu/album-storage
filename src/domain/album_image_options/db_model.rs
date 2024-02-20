use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Result;
use crate::schema::album_image;

#[derive(Queryable, Deserialize, Debug)]
#[diesel(table_name = album_image)]
pub struct AlbumImage {
    pub id: Uuid,
    pub album_id: Uuid,
    pub image_id: Uuid,
    pub order_index: i32,
    pub highlighted: bool,
    pub is_primary_album: bool,
}

#[derive(Insertable, Serialize, Debug)]
#[diesel(table_name = album_image)]
pub struct CreateAlbumImage {
    pub id: Uuid,
    pub album_id: Uuid,
    pub image_id: Uuid,
    pub order_index: i32,
    pub highlighted: bool,
    pub is_primary_album: bool,
}

#[derive(AsChangeset, Insertable, Serialize, Debug)]
#[diesel(table_name = album_image)]
pub struct UpdateAlbumImage {
    pub album_id: Option<Uuid>,
    pub order_index: Option<i32>,
    pub highlighted: Option<bool>,
    pub is_primary_album: Option<bool>,
}

pub struct AlbumImageDao;

impl AlbumImageDao {
    pub fn create(
        mm: &crate::db::ModelManager,
        create_album_image: &CreateAlbumImage,
    ) -> Result<AlbumImage> {
        let mut conn = mm.conn()?;

        diesel::insert_into(album_image::dsl::album_image)
            .values(create_album_image)
            .get_result::<AlbumImage>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn get_by_id(mm: &crate::db::ModelManager, id: &Uuid) -> Result<AlbumImage> {
        let mut conn = mm.conn()?;

        album_image::dsl::album_image
            .filter(album_image::dsl::id.eq(id))
            .first::<AlbumImage>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn get_by_image_id(
        mm: &crate::db::ModelManager,
        image_id: &Uuid,
    ) -> Result<Vec<AlbumImage>> {
        let mut conn = mm.conn()?;

        album_image::dsl::album_image
            .filter(album_image::dsl::image_id.eq(image_id))
            .load::<AlbumImage>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn update(
        mm: &crate::db::ModelManager,
        id: &Uuid,
        update_album_image: &UpdateAlbumImage,
    ) -> Result<AlbumImage> {
        let mut conn = mm.conn()?;

        diesel::update(album_image::dsl::album_image.find(id))
            .set(update_album_image)
            .get_result::<AlbumImage>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn update_many(
        mm: &crate::db::ModelManager,
        album_images: Vec<(Uuid, UpdateAlbumImage)>,
    ) -> Result<Vec<AlbumImage>> {
        let mut conn = mm.conn()?;

        conn.transaction(|conn| {
            let mut options = Vec::new();
            for (id, update_album_image) in album_images {
                let option = diesel::update(album_image::dsl::album_image.find(id))
                    .set(update_album_image)
                    .get_result::<AlbumImage>(conn)?;
                options.push(option);
            }
            Ok(options)
        })
    }
}
