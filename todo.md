[ ] 1. Resolve the conflict between sqlx(skip),sql(skip)

[x] 2. Make table attr add name attribute to Entity struct.

[x] 3. ✅ Add chrono feature to resolve all types of chrono
// Note. User can now provide custom factory functions for timestamps: #[sql(timestamp(created_at, chrono::Utc::now()))]
