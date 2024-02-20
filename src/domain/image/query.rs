use async_graphql::Object;
use async_graphql::*;
use async_graphql_relay::RelayNodeID;

use crate::graphql::{uuidIdentifiedQuery, AuthGuard, ConnectionResult, CursorParams};
use crate::{db::ModelManager, graphql::Error};

use super::Image;
use super::{DbImage, ImageDao};

#[derive(Default)]
pub struct ImageQuery;

#[Object]
impl ImageQuery {
    #[graphql(guard = "AuthGuard")]
    async fn image(&self, ctx: &Context<'_>, id: RelayNodeID<Image>) -> Result<Image> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };
        let image = ImageDao::get_by_id(mm, &id.to_uuid())
            .map(|image: DbImage| -> Image { image.into() })
            .map_err(|e| -> Error { e.into() })?;
        Ok(image)
    }

    #[graphql(guard = "AuthGuard")]
    async fn images(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> ConnectionResult<Image> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let images = ImageDao::list(mm)
            .map(|images: Vec<DbImage>| -> Vec<Image> {
                images.into_iter().map(|image| image.into()).collect()
            })
            .map_err(|e| -> Error { e.into() })?;

        uuidIdentifiedQuery(
            images.into_iter(),
            CursorParams::new(after, before, first, last),
            10,
        )
        .await
    }
}
