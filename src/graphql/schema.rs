use async_graphql::{Context, MergedObject, Schema, SimpleObject};
use async_graphql_relay::{RelayContext, RelayNodeInterface};
use reqwest::Client;
use uuid::Uuid;

use super::{error::Error, node::Node, uuid_cursor::Identifiable};
use crate::domain::{
    album::{AlbumMutation, AlbumQuery},
    album_image_options::AlbumImageMutation,
    image::{ImageMutation, ImageQuery, ImageSubscription},
    raw_album::{RawAlbumMutation, RawAlbumQuery},
};
use crate::{db::ModelManager, web::ctx::Ctx};

#[derive(Default)]
struct DefaultQuery;

#[derive(Default, SimpleObject, Clone, Debug)]
struct MyTest {
    value: i32,
    id: Uuid,
}

impl Identifiable for MyTest {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

#[async_graphql::Object]
impl DefaultQuery {
    async fn node(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(min_length = 38, max_length = 39))] id: String, // Ensure the length's of the longest 'node_suffix' plus 32 is validated.
    ) -> Result<Node, async_graphql::Error> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let ctx = RelayContext::new::<ModelManager>(mm.clone());
        Node::fetch_node(ctx, id).await
    }
    async fn hello(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let user_ctx = ctx.data_opt::<Ctx>();
        match user_ctx {
            Some(ctx) => Ok(format!("User logged in: {}", ctx.user_email)),
            None => Err(Error::AuthError.into()),
        }
    }
}

#[derive(Default)]
struct DefaultMutation;

#[async_graphql::Object]
impl DefaultMutation {
    async fn hello(&self, input: String) -> String {
        format!("{} world!", input)
    }
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(DefaultQuery, RawAlbumQuery, AlbumQuery, ImageQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    DefaultMutation,
    AlbumMutation,
    ImageMutation,
    AlbumImageMutation,
    RawAlbumMutation,
);

pub type WooBooSchema = Schema<QueryRoot, MutationRoot, ImageSubscription>;

pub fn create_schema(mm: ModelManager, req_client: Client) -> WooBooSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        ImageSubscription,
    )
    .data(mm)
    .data(req_client)
    .finish()
}
