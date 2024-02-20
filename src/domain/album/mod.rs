mod db_model;
mod graphql_model;
mod mutation;
mod query;

pub use db_model::{
    Album as DbAlbum, AlbumDao, CreateAlbum as DbCreateAlbum, UpdateAlbum as DbUpdateAlbum,
};
pub use graphql_model::Album;
pub use mutation::AlbumMutation;
pub use query::AlbumQuery;
