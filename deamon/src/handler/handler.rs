use config::{
    request::{Request, UserRequest},
    response::Response,
};
use sqlx::PgPool;

use crate::{
    confiig::Key,
    worker::worker::{
        secure::{default, lock, unlock},
        user::{add_user, remove_user, rename_user},
    },
};

pub async fn handle(req: Request, pool: PgPool, key: Key) -> Response {
    match req {
        Request::User(user_req) => match user_req {
            UserRequest::Add { name } => match add_user(name, pool).await {
                Ok(v) => Response::ok(v),
                Err(e) => Response::error(e),
            },
            UserRequest::Remove { name } => match remove_user(name, pool).await {
                Ok(v) => Response::ok(v),
                Err(e) => Response::error(e),
            },

            UserRequest::Rename { old_name, new_name } => {
                match rename_user(old_name, new_name, pool).await {
                    Ok(v) => Response::ok(v),
                    Err(e) => Response::error(e),
                }
            }
        },

        Request::Unlock(pass) => match unlock(pass, key).await {
            Ok(v) => Response::ok(v),
            Err(e) => Response::error(e),
        },
        Request::Lock => {
            let v = lock(key).await;
            Response::ok(v)
        }

        Request::Default(name) => match default(name, pool).await {
            Ok(v) => Response::ok(v),
            Err(e) => Response::error(e),
        },
    }
}
