# sqlorm

A developer-friendly, type-safe ORM for Rust built on top of SQLx, with compile-time checked queries, automatic CRUD, and relationship helpers.

1. [ ] Implement #[derive(Schema)]:
   - Parse #[table(schema = "...", name = "...")].
   - Generate fn table_name() -> &'static str.
2. [ ] Test macro with a dummy struct.

---

1. [ ] Generate .save() for INSERT + UPDATE.
2. [ ] Generate .delete() for DELETE.

---

1. [ ] Parse #[sql(fk -> Type, rename = "...")].
2. [ ] Generate .related_items() method.
3. [ ] Generate inverse relationship methods.

---
