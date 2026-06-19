use config::{
    error::VaultCliError,
    request::{Request, Vault, VaultAdd},
};

pub fn unlock() -> Result<String, VaultCliError> {
    let pass = rpassword::prompt_password("Please enter your master password: ")?;
    let re_pass = rpassword::prompt_password("Please re-enter your master password: ")?;

    if pass != re_pass {
        return Err(VaultCliError::AppError(
            "Passwords do not match".to_string(),
        ));
    }

    if pass.is_empty() {
        return Err(VaultCliError::AppError("Invalid password".to_string()));
    }

    Ok(pass)
}

pub fn add_pass(
    username: String,
    app: String,
    hint: String,
    master: Option<String>,
) -> Result<Request, VaultCliError> {
    let pass = rpassword::prompt_password("Please enter your App password: ")?;
    let re_pass = rpassword::prompt_password("Please re-enter your App password: ")?;

    if pass != re_pass {
        return Err(VaultCliError::AppError(
            "Passwords do not match".to_string(),
        ));
    }

    if pass.is_empty() {
        return Err(VaultCliError::AppError("Invalid password".to_string()));
    }

    Ok(Request::Vault(Vault::Add(VaultAdd {
        username,
        app,
        hint,
        master,
        pass,
    })))
}
