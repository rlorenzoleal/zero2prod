#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum SubscriptionStatus {
    PendingConfirmation,
    Confirmed,
}
