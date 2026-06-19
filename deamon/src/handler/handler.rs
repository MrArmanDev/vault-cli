use config::{
    request::{Request, UserRequest, Vault},
    response::{Password, Response},
};

use crate::{
    confiig::AppStates,
    worker::worker::{
        secure::{default, lock, unlock},
        user::{add_user, remove_user, rename_user},
        vault::{add_pass, get_pass},
    },
};

pub async fn handle(req: Request, state: AppStates) -> Response<Vec<Password>> {
    match req {
        Request::User(user_req) => match user_req {
            UserRequest::Add { name } => match add_user(name, &state.pool).await {
                Ok(v) => Response::ok(v),
                Err(e) => Response::error(e),
            },
            UserRequest::Remove { name } => match remove_user(name, &state.pool).await {
                Ok(v) => Response::ok(v),
                Err(e) => Response::error(e),
            },

            UserRequest::Rename { old_name, new_name } => {
                match rename_user(old_name, new_name, &state.pool).await {
                    Ok(v) => Response::ok(v),
                    Err(e) => Response::error(e),
                }
            }
        },

        Request::Unlock(pass) => match unlock(pass, &state.key).await {
            Ok(v) => Response::ok(v),
            Err(e) => Response::error(e),
        },
        Request::Lock => {
            let v = lock(&state.key).await;
            Response::ok(v)
        }

        Request::Default(name) => match default(name, &state.pool).await {
            Ok(v) => Response::ok(v),
            Err(e) => Response::error(e),
        },

        Request::Vault(v) => match v {
            Vault::Add(add) => match add_pass(add, &state).await {
                Ok(v) => Response::ok(v),
                Err(e) => Response::error(e),
            },

            Vault::Get(v) => match get_pass(v, &state).await {
                Ok(v) => {
                    Response { success: true, message: v.0, data: Some(v.1) }
                }

                Err(e)=> Response::error(e),
            },
        },
    }
}
