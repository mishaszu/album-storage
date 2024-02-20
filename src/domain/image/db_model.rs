use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{Error, ModelManager, Result};
use crate::domain::album::DbAlbum;
use crate::domain::album_image_options::DbCreateAlbumImage;
use crate::schema::{album, album_image, image};

#[derive(Queryable, Deserialize, Debug)]
#[diesel(table_name = image)]
pub struct Image {
    pub id: Uuid,
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

#[derive(Insertable, Serialize, Debug)]
#[diesel(table_name = image)]
pub struct CreateImage {
    pub id: Uuid,
    pub title: String,
    pub original_full_title: String,
    pub description: Option<String>,
    pub path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub is_uploaded: bool,
}

#[derive(AsChangeset, Insertable, Serialize, Debug)]
#[diesel(table_name = image)]
pub struct UpdateImage {
    pub title: Option<String>,
    pub original_full_title: Option<String>,
    pub description: Option<String>,
    pub path: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub is_uploaded: Option<bool>,
}

pub struct ImageDao;

impl ImageDao {
    pub fn create(mm: &ModelManager, new_image: CreateImage) -> Result<Image> {
        let mut conn = mm.conn()?;

        let res = diesel::insert_into(image::dsl::image)
            .values(&new_image)
            .get_result::<Image>(&mut conn)
            .expect("Error saving new image");

        Ok(res)
    }

    pub fn create_with_album(
        mm: &ModelManager,
        album_id: &Uuid,
        new_image: CreateImage,
        order_index: Option<i32>,
        is_primary_album: bool,
    ) -> Result<Image> {
        let mut conn = mm.conn()?;

        conn.transaction(|conn| {
            let res = diesel::insert_into(image::dsl::image)
                .values(&new_image)
                .returning(image::all_columns)
                .get_result::<Image>(conn)
                .map_err(|e| -> Error { e.into() })?;

            let size = match order_index {
                Some(order_index) => order_index,
                None => album_image::dsl::album_image
                    .filter(album_image::dsl::album_id.eq(album_id))
                    .count()
                    .get_result::<i64>(conn)
                    .map(|c| c as i32)
                    .map_err(|e| -> Error { e.into() })?,
            };

            let new_album_image = DbCreateAlbumImage {
                id: Uuid::new_v4(),
                album_id: *album_id,
                image_id: new_image.id,
                order_index: size,
                highlighted: false,
                is_primary_album,
            };

            diesel::insert_into(album_image::dsl::album_image)
                .values(&new_album_image)
                .execute(conn)?;

            Ok(res)
        })
    }

    pub fn get_many_by_ids(mm: &ModelManager, ids: Vec<Uuid>) -> Result<Vec<Image>> {
        let mut conn = mm.conn()?;

        image::dsl::image
            .filter(image::dsl::id.eq_any(ids))
            .load::<Image>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn get_by_id(mm: &ModelManager, id: &Uuid) -> Result<Image> {
        let mut conn = mm.conn()?;

        image::dsl::image
            .find(id)
            .first(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn get_by_album_id(mm: &ModelManager, album_id: &Uuid) -> Result<Vec<Image>> {
        let mut conn = mm.conn()?;

        album_image::dsl::album_image
            .filter(album_image::dsl::album_id.eq(album_id))
            .inner_join(image::dsl::image)
            .select(image::all_columns)
            .load::<Image>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn list(mm: &ModelManager) -> Result<Vec<Image>> {
        let mut conn = mm.conn()?;

        image::dsl::image
            .load::<Image>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn update(mm: &ModelManager, id: &Uuid, update_image: UpdateImage) -> Result<Image> {
        let mut conn = mm.conn()?;

        diesel::update(image::dsl::image.find(id))
            .set(&update_image)
            .get_result::<Image>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn delete(mm: &ModelManager, id: &Uuid) -> Result<Image> {
        let mut conn = mm.conn()?;

        diesel::delete(image::dsl::image.find(id))
            .get_result::<Image>(&mut conn)
            .map_err(|e| e.into())
    }

    pub fn is_uploaded(mm: &ModelManager, original_full_title: &str) -> Result<bool> {
        let mut conn = mm.conn()?;

        let res = image::dsl::image
            .filter(image::dsl::original_full_title.eq(original_full_title))
            .first::<Image>(&mut conn)
            .map(|v| v.is_uploaded)
            .map_err(|e| e.into());

        match res {
            Ok(v) => Ok(v),
            Err(e) => match e {
                Error::DbEntityNotFound => Ok(false),
                _ => Err(e),
            },
        }
    }

    pub fn get_albums(mm: &ModelManager, image_id: &Uuid) -> Result<Vec<DbAlbum>> {
        let mut conn = mm.conn()?;

        album_image::dsl::album_image
            .filter(album_image::dsl::image_id.eq(image_id))
            .inner_join(album::dsl::album)
            .select(album::all_columns)
            .load::<DbAlbum>(&mut conn)
            .map_err(|e| e.into())
    }
}
