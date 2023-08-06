use crate::configuration::config::Config;
use crate::errors::bad_request::BadRequest;
use crate::errors::internal_server_error::InternalServerError;
use crate::repository::user::user_model::User;
use crate::web::controller::user::user_controller::ConvertError;
use crate::web::dto::authentication::login_request::LoginRequest;
use crate::web::dto::authentication::login_response::LoginResponse;
use crate::web::dto::authentication::register_request::RegisterRequest;
use crate::web::dto::permission::permission_dto::SimplePermissionDto;
use crate::web::dto::role::role_dto::SimpleRoleDto;
use crate::web::dto::user::user_dto::SimpleUserDto;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use log::error;

/// # Summary
///
/// Convert a User into a SimpleUserDto
///
/// # Arguments
///
/// * `user` - A User
///
/// # Example
///
/// ```
/// let user = User::new("user1".to_string(), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None);
/// let user_dto = convert_user_to_simple_dto(user);
/// ```
///
/// # Returns
///
/// * `Result<SimpleUserDto, ConvertError>` - The result containing the SimpleUserDto or the ConvertError that occurred
async fn convert_user_to_simple_dto(
    user: User,
    pool: &Config,
) -> Result<SimpleUserDto, ConvertError> {
    let mut user_dto = SimpleUserDto::from(user.clone());

    if user.roles.is_some() {
        let roles = match pool
            .services
            .role_service
            .find_by_id_vec(user.roles.clone().unwrap(), &pool.database)
            .await
        {
            Ok(d) => d,
            Err(e) => {
                return Err(ConvertError::RoleError(e));
            }
        };

        if !roles.is_empty() {
            let mut role_dto_list: Vec<SimpleRoleDto> = vec![];

            for r in &roles {
                let mut role_dto = SimpleRoleDto::from(r.clone());
                if r.permissions.is_some() {
                    let mut permission_dto_list: Vec<SimplePermissionDto> = vec![];
                    let permissions = match pool
                        .services
                        .permission_service
                        .find_by_id_vec(r.permissions.clone().unwrap(), &pool.database)
                        .await
                    {
                        Ok(d) => d,
                        Err(e) => return Err(ConvertError::PermissionError(e)),
                    };

                    if !permissions.is_empty() {
                        for p in permissions {
                            permission_dto_list.push(SimplePermissionDto::from(p));
                        }
                    }

                    if !permission_dto_list.is_empty() {
                        role_dto.permissions = Some(permission_dto_list)
                    }
                }

                role_dto_list.push(role_dto);
            }

            user_dto.roles = Some(role_dto_list);
        }
    }

    Ok(user_dto)
}

#[post("/login")]
pub async fn login(
    login_request: web::Json<LoginRequest>,
    pool: web::Data<Config>,
) -> HttpResponse {
    if login_request.username.is_empty() {
        return HttpResponse::BadRequest().json("Username is required");
    }
    if login_request.password.is_empty() {
        return HttpResponse::BadRequest().json("Password is required");
    }

    let user = match pool
        .services
        .user_service
        .find_by_username(&login_request.username, &pool.database)
        .await
    {
        Ok(u) => match u {
            Some(user) => user,
            None => {
                return HttpResponse::BadRequest().finish();
            }
        },
        Err(e) => {
            error!("Failed to find user by email: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    let salt = match SaltString::from_b64(&pool.salt) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to generate salt: {}", e);
            return HttpResponse::InternalServerError()
                .json(InternalServerError::new("Failed to generate salt"));
        }
    };

    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(login_request.password.as_bytes(), &salt) {
        Ok(e) => e.to_string(),
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return HttpResponse::InternalServerError()
                .json(InternalServerError::new("Failed to hash password"));
        }
    };

    if password_hash != user.password || !user.enabled {
        return HttpResponse::BadRequest().finish();
    }

    match pool.services.jwt_service.generate_jwt_token(&user.email) {
        Some(t) => HttpResponse::Ok().json(LoginResponse::new(t)),
        None => HttpResponse::InternalServerError()
            .json(InternalServerError::new("Failed to generate JWT token")),
    }
}

#[post("/register")]
pub async fn register(
    register_request: web::Json<RegisterRequest>,
    pool: web::Data<Config>,
) -> HttpResponse {
    if register_request.username.is_empty() {
        return HttpResponse::BadRequest().json(BadRequest::new("Empty usernames are not allowed"));
    }

    if register_request.password.is_empty() {
        return HttpResponse::BadRequest().json(BadRequest::new("Empty passwords are not allowed"));
    }

    if register_request.email.is_empty() {
        return HttpResponse::BadRequest()
            .json(BadRequest::new("Empty email addresses are not allowed"));
    }

    let register_request = register_request.into_inner();

    let default_roles: Option<Vec<String>> = match pool
        .services
        .role_service
        .find_by_name("DEFAULT", &pool.database)
        .await
    {
        Ok(r) => match r {
            Some(role) => Some(vec![role.id]),
            None => None,
        },
        Err(e) => {
            error!("Failed to find default role: {}", e);
            return HttpResponse::InternalServerError()
                .json(InternalServerError::new(&e.to_string()));
        }
    };

    let mut user = User::from(register_request);

    let password = &user.password.as_bytes();
    let salt = match SaltString::from_b64(&pool.salt) {
        Ok(s) => s,
        Err(e) => {
            error!("Error generating salt: {}", e);
            return HttpResponse::InternalServerError()
                .json(InternalServerError::new("Failed to generate salt"));
        }
    };

    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(password, &salt) {
        Ok(e) => e.to_string(),
        Err(e) => {
            error!("Error hashing password: {}", e);
            return HttpResponse::InternalServerError()
                .json(InternalServerError::new("Failed to hash password"));
        }
    };

    user.password = password_hash;
    user.roles = default_roles;

    match pool
        .services
        .user_service
        .create(user, &pool.database)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!("Error creating User: {}", e);
            HttpResponse::InternalServerError().json(InternalServerError::new(&e.to_string()))
        }
    }
}

#[get("/current")]
pub async fn current_user(req: HttpRequest, pool: web::Data<Config>) -> HttpResponse {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let username = match pool.services.jwt_service.verify_jwt_token(token) {
                    Ok(user) => user,
                    Err(e) => {
                        error!("Failed to verify JWT token: {}", e);
                        return HttpResponse::Forbidden().finish();
                    }
                };

                let user = match pool
                    .services
                    .user_service
                    .find_by_email(&username, &pool.database)
                    .await
                {
                    Ok(u) => match u {
                        Some(user) => user,
                        None => {
                            return HttpResponse::Forbidden().finish();
                        }
                    },
                    Err(e) => {
                        error!("Failed to find user by email: {}", e);
                        return HttpResponse::Forbidden().finish();
                    }
                };

                return match convert_user_to_simple_dto(user, &pool).await {
                    Ok(u) => HttpResponse::Ok().json(u),
                    Err(e) => {
                        error!("Failed to convert User to SimpleUserDto: {}", e);
                        HttpResponse::Forbidden().finish()
                    }
                };
            }
        }
    }

    HttpResponse::Forbidden().finish()
}
