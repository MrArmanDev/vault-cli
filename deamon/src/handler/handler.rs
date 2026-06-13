use config::{
    request::{Request, UserRequest},
    response::Response,
};
use sqlx::PgPool;

use crate::worker::worker::add_user;

pub async fn handle(req: Request, pool: PgPool) -> Response {
    match req {
        Request::User(user_req) => match user_req {
            UserRequest::Add { name } => match add_user(name, pool).await {
                Ok(v) => Response {
                    success: true,
                    message: v,
                    data: None,
                },

                Err(e) => Response {
                    success: false,
                    message: format!("{:?}", e),
                    data: None,
                },
            },
            _ => todo!()
        },
    }
}
