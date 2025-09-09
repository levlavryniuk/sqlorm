TODO

[x] Auto-generated lazy queries
[x] Auto-generated eager queries

- Query builder e.g. User::query().with_jars().fetch_one();
  [x] Automatic generation of traits and impls for relations
  [x] Make it smart. Switch between batch and eager fetch methods depending on relation.kind

[x] Column selector for queries
[ ] Renaming fields (e.g. sqlx(rename))
[x] Add support for flexible WHERE conditions. Try to make it type safe.
[?] Deal with prelude Relation traits exports
[x] Make fetches accept impl Acquire instead of &PgPool
