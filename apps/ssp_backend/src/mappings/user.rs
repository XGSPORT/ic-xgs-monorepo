use crate::repositories::User;

impl From<User> for ssp_backend_types::User {
    fn from(user: User) -> Self {
        Self {
            sub: user.jwt_sub,
            db_id: user.db_id.to_string(),
            created_at: user.created_at.to_string(),
        }
    }
}
