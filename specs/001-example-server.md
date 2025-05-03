# Example Server

Use `axum` to create an example server of user system.
## Requirements

- Create a simple user system with the following endpoints:
  - `POST /users` - create a user
  - `GET /users/:id` - get user by id
  - `GET /users` - list users
  - `PUT /users/:id` - update user
  - `DELETE /users/:id` - delete user

## Data Model

User schema:
```rust
struct User {
    id: u64,
    name: String,
    email: String,
    #[serde(skip_serializing)]
    password: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

App state:
Use `Dashmap` to store the users in memory, use `AtomicU64` to generate the id of the user.
```rust
struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    argon2: Argon2<'static>,
    next_id: AtomicU64,
    users: DashMap<u64, User>,
}
```

App state should have the following methods:
- `new`: initialize the app state
- `create_user`: create a new user
- `get_user`: get a user by id
- `list_users`: list all users
- `update_user`: update a user
- `delete_user`: delete a user
- `health`: check the health of the app
## Dependencies

- `axum` 0.8
- `dashmap` 6.1
- `tokio` 1.44
- `serde` 1.0
- `argon2` 0.5
