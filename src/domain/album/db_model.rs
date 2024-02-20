use diesel::prelude::*;
use diesel::{deserialize::Queryable, ExpressionMethods, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{ModelManager, Result};
use crate::domain::image::DbImage;
use crate::schema::{album, album_image, image};

#[derive(Queryable, Deserialize, Debug)]
#[diesel(table_name = album)]
pub struct Album {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub original_title: String,
    pub is_uploaded: bool,
    pub prev_image_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Serialize, Debug)]
#[diesel(table_name = album)]
pub struct CreateAlbum {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub original_title: String,
}

#[derive(AsChangeset, Insertable, Serialize, Debug)]
#[diesel(table_name = album)]
pub struct UpdateAlbum {
    pub title: Option<String>,
    pub description: Option<String>,
    pub original_title: Option<String>,
    pub is_uploaded: Option<bool>,
    pub prev_image_id: Option<Uuid>,
}

pub struct AlbumDao;

impl AlbumDao {
    pub fn create(mm: &ModelManager, new_album: CreateAlbum) -> Result<Album> {
        let mut conn = mm.conn()?;

        let res = diesel::insert_into(album::dsl::album)
            .values(&new_album)
            .get_result::<Album>(&mut conn)
            .expect("Error saving new album");

        Ok(res)
    }

    pub fn get_by_id(mm: &ModelManager, id: &Uuid) -> Result<Album> {
        let mut conn = mm.conn()?;

        album::dsl::album
            .filter(album::dsl::id.eq(id))
            .first::<Album>(&mut conn)
            .map_err(Into::into)
    }

    pub fn list(mm: &ModelManager) -> Result<Vec<Album>> {
        let mut conn = mm.conn()?;

        album::dsl::album
            .load::<Album>(&mut conn)
            .map_err(Into::into)
    }

    pub fn update(mm: &ModelManager, id: &Uuid, update_album: UpdateAlbum) -> Result<Album> {
        let mut conn = mm.conn()?;

        let res = diesel::update(album::dsl::album.filter(album::dsl::id.eq(id)))
            .set(&update_album)
            .get_result::<Album>(&mut conn)
            .expect("Error updating album");

        Ok(res)
    }

    pub fn delete(mm: &ModelManager, id: &Uuid) -> Result<usize> {
        let mut conn = mm.conn()?;

        let res = diesel::delete(album::dsl::album.filter(album::dsl::id.eq(id)))
            .execute(&mut conn)
            .expect("Error deleting album");

        Ok(res)
    }

    pub fn get_album_images(mm: &ModelManager, album_id: &Uuid) -> Result<Vec<DbImage>> {
        let mut conn = mm.conn()?;

        album_image::dsl::album_image
            .filter(album_image::dsl::album_id.eq(album_id))
            .inner_join(image::dsl::image)
            .select(image::all_columns)
            .load::<DbImage>(&mut conn)
            .map_err(Into::into)
    }

    pub fn get_album_images_sorted(mm: &ModelManager, album_id: &Uuid) -> Result<Vec<DbImage>> {
        let mut conn = mm.conn()?;

        let mut images = album_image::dsl::album_image
            .filter(album_image::dsl::album_id.eq(album_id))
            .inner_join(image::dsl::image)
            .select((album_image::order_index, image::all_columns))
            .load::<(i32, DbImage)>(&mut conn)?;

        images.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(images.into_iter().map(|(_, image)| image).collect())
    }
}
