use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    #[serde(skip_serializing)]
    password: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

#[derive(Clone)]
struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    argon2: Argon2<'static>,
    next_id: AtomicU64,
    users: DashMap<u64, User>,
}

impl AppState {
    fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                argon2: Argon2::default(),
                next_id: AtomicU64::new(1),
                users: DashMap::new(),
            }),
        }
    }

    async fn create_user(&self, req: CreateUserRequest) -> Result<User, StatusCode> {
        info!("create_user, req: {req:?}");
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .inner
            .argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .to_string();

        let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now();
        let user = User {
            id,
            name: req.name,
            email: req.email,
            password: password_hash,
            created_at: now,
            updated_at: now,
        };

        self.inner.users.insert(id, user.clone());
        Ok(user)
    }

    async fn get_user(&self, id: u64) -> Result<User, StatusCode> {
        self.inner
            .users
            .get(&id)
            .map(|user| user.clone())
            .ok_or(StatusCode::NOT_FOUND)
    }

    async fn list_users(&self) -> Vec<User> {
        self.inner.users.iter().map(|entry| entry.clone()).collect()
    }

    async fn update_user(&self, id: u64, req: UpdateUserRequest) -> Result<User, StatusCode> {
        let mut user = self.inner.users.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;

        if let Some(name) = req.name {
            user.name = name;
        }
        if let Some(email) = req.email {
            user.email = email;
        }
        if let Some(password) = req.password {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = self
                .inner
                .argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .to_string();
            user.password = password_hash;
        }
        user.updated_at = Utc::now();
        Ok(user.clone())
    }

    async fn delete_user(&self, id: u64) -> Result<(), StatusCode> {
        self.inner
            .users
            .remove(&id)
            .map(|_| ())
            .ok_or(StatusCode::NOT_FOUND)
    }

    async fn health(&self) -> Result<(), StatusCode> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app_state = AppState::new();
    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
        .route("/users/{id}", put(update_user))
        .route("/users/{id}", delete(delete_user))
        .route("/health", get(health))
        .with_state(app_state);

    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    match state.create_user(req).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(status) => status.into_response(),
    }
}

async fn get_user(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    match state.get_user(id).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(status) => status.into_response(),
    }
}

async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let users = state.list_users().await;
    (StatusCode::OK, Json(users)).into_response()
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match state.update_user(id, req).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(status) => status.into_response(),
    }
}

async fn delete_user(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    match state.delete_user(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(status) => status.into_response(),
    }
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    state.health().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_create_user() {
        let state = AppState::new();
        let req = CreateUserRequest {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = state.create_user(req).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert!(!user.password.is_empty());
    }

    #[tokio::test]
    async fn test_get_user() {
        let state = AppState::new();
        let req = CreateUserRequest {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let created_user = state.create_user(req).await.unwrap();
        let retrieved_user = state.get_user(created_user.id).await;
        assert!(retrieved_user.is_ok());
        let user = retrieved_user.unwrap();
        assert_eq!(user.id, created_user.id);
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_nonexistent_user() {
        let state = AppState::new();
        let result = state.get_user(999).await;
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_users() {
        let state = AppState::new();

        // Create multiple users
        for i in 0..3 {
            let req = CreateUserRequest {
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
                password: "password123".to_string(),
            };
            state.create_user(req).await.unwrap();
        }

        let users = state.list_users().await;
        assert_eq!(users.len(), 3);
    }

    #[tokio::test]
    async fn test_update_user() {
        let state = AppState::new();
        let req = CreateUserRequest {
            name: "Original Name".to_string(),
            email: "original@example.com".to_string(),
            password: "password123".to_string(),
        };

        let created_user = state.create_user(req).await.unwrap();
        let update_req = UpdateUserRequest {
            name: Some("Updated Name".to_string()),
            email: Some("updated@example.com".to_string()),
            password: Some("newpassword123".to_string()),
        };

        let updated_user = state.update_user(created_user.id, update_req).await;
        assert!(updated_user.is_ok());
        let user = updated_user.unwrap();
        assert_eq!(user.name, "Updated Name");
        assert_eq!(user.email, "updated@example.com");
        assert!(!user.password.is_empty());
    }

    #[tokio::test]
    async fn test_update_nonexistent_user() {
        let state = AppState::new();
        let update_req = UpdateUserRequest {
            name: Some("New Name".to_string()),
            email: None,
            password: None,
        };

        let result = state.update_user(999, update_req).await;
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let state = AppState::new();
        let req = CreateUserRequest {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let created_user = state.create_user(req).await.unwrap();
        let delete_result = state.delete_user(created_user.id).await;
        assert!(delete_result.is_ok());

        // Verify user is deleted
        let get_result = state.get_user(created_user.id).await;
        assert_eq!(get_result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_user() {
        let state = AppState::new();
        let result = state.delete_user(999).await;
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_health() {
        let state = AppState::new();
        let result = state.health().await;
        assert!(result.is_ok());
    }
}
