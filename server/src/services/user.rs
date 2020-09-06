use crate::models::auth::{
    PasswordToken, PasswordTokenRepository, SignUpToken, SignUpTokenRepository,
};
use crate::models::error::{get_service_error, ServiceError};
use crate::models::user::*;
use crate::models::user_key::UserKeyRepository;
use crate::utils::password_util;

pub struct UserService {}

impl UserService {
    /// Finds a user by id.
    pub fn get_one(id: u64) -> Result<UserDTO, ServiceError> {
        let user = {
            let user_repository = UserRepository::new();
            user_repository.find_by_id(id)?
        };

        Ok(UserDTO {
            id: user.id,
            name: user.name,
            email: user.email,
            avatar_url: user.avatar_url,
            updated_at: user.updated_at,
            created_at: user.created_at,
        })
    }

    /// Finds all users.
    pub fn get_list() -> Result<Vec<UserDTO>, ServiceError> {
        let user_list = {
            let user_repository = UserRepository::new();
            user_repository.find_all()?
        };

        Ok(user_list
            .iter()
            .map(|user| -> UserDTO {
                UserDTO {
                    id: user.id,
                    name: user.name.clone(),
                    email: user.email.clone(),
                    avatar_url: user.avatar_url.clone(),
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                }
            })
            .collect())
    }

    /// Creates a new user.
    ///
    /// 1. Finds serialized token by token key from arguments.
    /// 2. Deserializes the found token and compares pin from token and it from arguments.
    /// 3. If the pins are equal, deletes the token from redis and creates a new user.
    pub fn create(
        user_public_key: &str,
        token_key: &str,
        token_pin: &str,
    ) -> Result<bool, ServiceError> {
        let token: SignUpToken = {
            let mut token_repository = SignUpTokenRepository::new();
            let serialized_token = token_repository.find(token_key)?;

            let deserialized_token: SignUpToken =
                if let Ok(deserialized_token) = serde_json::from_str(&serialized_token) {
                    deserialized_token
                } else {
                    return Err(get_service_error(ServiceError::InvalidFormat));
                };

            if token_pin == deserialized_token.pin {
                let _ = token_repository.delete(token_key)?;
                deserialized_token
            } else {
                return Err(get_service_error(ServiceError::Unauthorized));
            }
        };

        let user = {
            let user_repository = UserRepository::new();

            user_repository.create(
                &token.name,
                &token.email,
                &token.password,
                &token.avatar_url,
            )?;

            user_repository.find_by_email(&token.email)?
        };

        let user_key_repository = UserKeyRepository::new();
        user_key_repository.create(user.id, user_public_key)
    }

    /// Deletes a user.
    pub fn delete(id: u64) -> Result<bool, ServiceError> {
        let user_repository = UserRepository::new();
        user_repository.delete(id)
    }

    /// Updates a new user.
    pub fn update(
        id: u64,
        name: &Option<String>,
        password: &Option<String>,
        avatar_url: &Option<String>,
    ) -> Result<bool, ServiceError> {
        if name.is_none() && password.is_none() && avatar_url.is_none() {
            return Err(get_service_error(ServiceError::InvalidArgument));
        }

        if let (Some(name), Some(password), Some(avatar_url)) = (name, password, avatar_url) {
            if name.trim().is_empty() || password.trim().is_empty() || avatar_url.trim().is_empty()
            {
                return Err(get_service_error(ServiceError::InvalidArgument));
            }
        }

        let hashed_password = if let Some(password) = password {
            Some(password_util::get_hashed_password(&password))
        } else {
            None
        };

        let user_repository = UserRepository::new();
        user_repository.update(id, name, &hashed_password, avatar_url)
    }

    // Reset the password.
    pub fn reset_password(
        email: &str,
        token_id: &str,
        temporary_password: &str,
        new_password: &str,
    ) -> Result<bool, ServiceError> {
        let user_repository = UserRepository::new();
        let user = user_repository.find_by_email(email)?;

        let mut token_repository = PasswordTokenRepository::new(user.id);
        let token: PasswordToken = {
            let serialized_token = token_repository.find()?;
            if let Ok(deserialized_token) = serde_json::from_str(&serialized_token) {
                deserialized_token
            } else {
                return Err(get_service_error(ServiceError::InvalidFormat));
            }
        };

        if token.id == token_id && token.password == temporary_password {
            let hashed_password = password_util::get_hashed_password(new_password);
            user_repository.update(user.id, &None, &Some(hashed_password), &None)?;
            token_repository.delete()
        } else {
            Err(get_service_error(ServiceError::UserNotFound(
                email.to_string(),
            )))
        }
    }
}
