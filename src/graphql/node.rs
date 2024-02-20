use async_graphql::Interface;
use async_graphql_relay::RelayInterface;

use crate::domain::{album::Album, album_image_options::AlbumImage, image::Image};

#[derive(Interface, RelayInterface)]
#[graphql(field(name = "id", ty = "NodeGlobalID"))] // The 'NodeGlobalID' type comes from the 'RelayInterface' macro.
pub enum Node {
    Album(Album),
    Image(Image),
    AlbumImage(AlbumImage),
}
