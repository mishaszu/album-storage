use async_graphql::{Context, Guard, Result};

use crate::{
    db::ModelManager, domain::user::db_model::UserBmc, graphql::error::Error, web::ctx::Ctx,
};

pub struct AuthGuard;

#[async_trait::async_trait]
impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let app_ctx = ctx.data_opt::<Ctx>();
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let user_account_id = match app_ctx {
            Some(ctx) => ctx.user_id,
            None => return Err(Error::AccessError("No user logged in".to_string()).into()),
        };

        let user = UserBmc::get_by_id(mm, &user_account_id).map_err(Error::DbError);

        match user {
            Err(_) => Err(Error::AccessError(user_account_id.to_string()).into()),
            Ok(_) => Ok(()),
        }
    }
}
