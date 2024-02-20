use async_graphql::Result;
use async_graphql::*;
use futures_util::stream::Stream;
use imagesize::{blob_size, ImageSize};
use reqwest::Client;
use serde::Serialize;
use tracing::error;
use uuid::Uuid;

use crate::config::config;
use crate::db::ModelManager;
use crate::services::lust::{Lust, LustResponse};
use crate::utils::{delete_file, read_file};

use super::Image;
use super::{DbCreateImage, ImageDao};
use crate::graphql::{AuthGuard, Error};

#[derive(Default)]
pub struct ImageSubscription;

#[derive(SimpleObject, Serialize)]
struct DeletionResult {
    image: String,
    success: bool,
}

#[Subscription]
impl ImageSubscription {
    #[graphql(guard = "AuthGuard")]
    async fn upload_album_images<'a>(
        &'a self,
        ctx: &'a Context<'a>,
        images: Vec<String>,
        album_id: Uuid,
        album_path: String,
        is_primary_album: bool,
    ) -> Result<impl Stream<Item = Result<Option<Image>>> + 'a> {
        let client = ctx.data_opt::<Client>();
        let client = match client {
            Some(client) => client,
            None => return Err(Error::ClientNotInContext.into()),
        };
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };
        let mut images = images;
        images.sort();
        let stream = async_stream::stream! {
            for image in images {
                let file = read_file(&format!("{}/{}", &album_path, &image))
                    .await
                    .map_err(|_e| -> async_graphql::Error { Error::FailedToReadFile.into() });
                let ( res, dimensions, size ): ( Result<LustResponse>, Option<ImageSize>, Option<usize> ) = match file {
                    Ok(file) => {
                        let image_dimensions = blob_size(&file).map_err(|_| Error::BadImage);
                        let size = file.len();
                        let res = Lust::post_file(&client, &config().LUST_BUCKET, file).await.map_err(|e| -> async_graphql::Error { e.into()});

                        (res, image_dimensions.ok(), Some(size))
                    },
                    Err(e) => {
                        (Err(e), None, None)
                    }
                };
                match ( res, dimensions, size ) {
                    (Ok(res), Some(dimensions), Some(_)) => {

                        let full_path = format!("{}/{}", album_path, image);

                        let is_image_uploaded = ImageDao::is_uploaded(mm, &full_path)
                            .map_err(|e| -> Error { e.into() })?;

                        match is_image_uploaded {
                            true => {
                                yield Err(Error::EntityExists.into());
                                continue;
                            }
                            false => {
                                let image_res = ImageDao::create_with_album(
                                    mm,
                                    &album_id,
                                    DbCreateImage {
                                        id: Uuid::new_v4(),
                                        title: image.clone(),
                                        description: None,
                                        original_full_title: full_path.clone(),
                                        path: res.image_id,
                                        width: Some(dimensions.width as i32),
                                        height: Some(dimensions.height as i32),
                                        is_uploaded: true,
                                    },
                                    None,
                                    is_primary_album,
                                ).map_err(|e| -> Error { e.into() });
                                // TODO: delete original image destination
                                match image_res {
                                    Ok(image_res) => {
                                        let image: Image = image_res.into();
                                        let delete_result = delete_file(&full_path).await;
                                        match delete_result {
                                            Ok(_) => () ,
                                            Err(e) => {
                                            error!("Failed to delete original file: {:?}", e);
                                            }
                                        }
                                        yield Ok(Some(image));
                                    }
                                    Err(e)=> {
                                        yield Err(e.into());
                                    }
                                }
                            },
                        }
                    }
                    (Err(e), _, _) => {
                        yield Err(e);
                    }
                    (_, _, _) => {
                        yield Err(Error::BadImage.into());
                    }
                }
            }
        };
        Ok(stream)
    }

    #[graphql(guard = "AuthGuard")]
    async fn delete_album_images<'a>(
        &'a self,
        ctx: &'a Context<'a>,
        album_id: Uuid,
    ) -> Result<impl Stream<Item = Result<DeletionResult>> + 'a> {
        let client = ctx.data_opt::<Client>();
        let client = match client {
            Some(client) => client,
            None => return Err(Error::ClientNotInContext.into()),
        };
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let images = ImageDao::get_by_album_id(mm, &album_id).map_err(|e| -> Error { e.into() })?;

        let stream = async_stream::stream! {
            for image in images {
                let image_id = image.id;
                let image_path = image.path.clone();
                let delete_result = Lust::delete_file(&client, &config().LUST_BUCKET, &image_path).await;

                match delete_result {
                    Ok(_) => {
                        let delete_result = ImageDao::delete(mm, &image_id).map_err(|e| -> Error { e.into() });
                        match delete_result {
                            Ok(_) => {
                                yield Ok(DeletionResult {
                                    image: image_path,
                                    success: true,
                                });
                            }
                            Err(e) => {
                                yield Err(e.into());
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(e.into());
                    }
                }
            }
        };
        Ok(stream)
    }

    #[graphql(guard = "AuthGuard")]
    async fn delete_images<'a>(
        &'a self,
        ctx: &'a Context<'a>,
        images: Vec<Uuid>,
    ) -> Result<impl Stream<Item = Result<DeletionResult>> + 'a> {
        let client = ctx.data_opt::<Client>();
        let client = match client {
            Some(client) => client,
            None => return Err(Error::ClientNotInContext.into()),
        };
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let images = ImageDao::get_many_by_ids(mm, images).map_err(|e| -> Error { e.into() })?;

        let stream = async_stream::stream! {
            for image in images {
                let image_id = image.id;
                let image_path = image.path.clone();
                let delete_result = Lust::delete_file(&client, &config().LUST_BUCKET, &image_path).await;

                match delete_result {
                    Ok(_) => {
                        let delete_result = ImageDao::delete(mm, &image_id).map_err(|e| -> Error { e.into() });
                        match delete_result {
                            Ok(_) => {
                                yield Ok(DeletionResult {
                                    image: image_path,
                                    success: true,
                                });
                            }
                            Err(e) => {
                                yield Err(e.into());
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(e.into());
                    }
                }
            }
        };
        Ok(stream)
    }
}
