use sqlorm_core::TableInfo;
use sqlorm_core::qb::{Column, JoinSpec, JoinType, QB};
use std::marker::PhantomData;

fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn select_and_from_sql() {
    let base = TableInfo {
        name: "users",
        alias: "u".to_string(),
        columns: vec!["id", "name"],
    };
    let qb = QB::<()>::new(base);
    let sql = normalize(&qb.to_sql());
    assert_eq!(
        sql,
        "SELECT u.id AS u__id, u.name AS u__name FROM \"users\" AS u"
    );
}

#[test]
fn join_sql() {
    let base = TableInfo {
        name: "users",
        alias: "u".to_string(),
        columns: vec!["id"],
    };
    let foreign = TableInfo {
        name: "profiles",
        alias: "p".to_string(),
        columns: vec!["user_id", "bio"],
    };
    let join = JoinSpec {
        join_type: JoinType::Left,
        relation_name: "profile",
        foreign_table: foreign,
        on: ("id", "user_id"),
    };
    let qb = QB::<()>::new(base).join_eager(join);
    let sql = normalize(&qb.to_sql());
    assert_eq!(
        sql,
        "SELECT u.id AS u__id, p.user_id AS p__user_id, p.bio AS p__bio \
         FROM \"users\" AS u \
         LEFT JOIN \"profiles\" AS p ON u.id = p.user_id"
    );
}

#[test]
fn single_filter_sql() {
    let base = TableInfo {
        name: "users",
        alias: "u".to_string(),
        columns: vec!["id"],
    };
    let col = Column::<i32> {
        name: "id",
        table_alias: "u",
        aliased_name: "u__id",
        _marker: PhantomData,
    };
    let qb = QB::<()>::new(base).filter(col.eq(7));
    let sql = normalize(&qb.to_sql());
    #[cfg(feature = "postgres")]
    assert_eq!(
        sql,
        "SELECT u.id AS u__id FROM \"users\" AS u WHERE u.id = $1"
    );
    #[cfg(feature = "sqlite")]
    assert_eq!(
        sql,
        "SELECT u.id AS u__id FROM \"users\" AS u WHERE u.id = ?"
    );
}

#[test]
fn multiple_filters_and_in_sql() {
    let base = TableInfo {
        name: "users",
        alias: "u".to_string(),
        columns: vec!["id", "name"],
    };
    let id = Column::<i32> {
        name: "id",
        table_alias: "u",
        aliased_name: "u__id",
        _marker: PhantomData,
    };
    let name = Column::<String> {
        name: "name",
        table_alias: "u",
        aliased_name: "u__name",
        _marker: PhantomData,
    };
    let qb = QB::<()>::new(base)
        .filter(id.gt(10))
        .filter(name.in_(vec!["a".to_string(), "b".to_string()]))
        .filter(name.like("c%".to_string()));
    let sql = normalize(&qb.to_sql());
    #[cfg(feature = "postgres")]
    assert_eq!(
        sql,
        "SELECT u.id AS u__id, u.name AS u__name \
         FROM \"users\" AS u \
         WHERE u.id > $1 AND u.name IN ($2, $3) AND u.name LIKE $4"
    );
    #[cfg(feature = "sqlite")]
    assert_eq!(
        sql,
        "SELECT u.id AS u__id, u.name AS u__name \
         FROM \"users\" AS u \
         WHERE u.id > ? AND u.name IN (?, ?) AND u.name LIKE ?"
    );
}

#[test]
#[should_panic(
    expected = "Cannot select empty column list. At least one column must be specified."
)]
fn select_empty_columns_panics() {
    let base = TableInfo {
        name: "users",
        alias: "u".to_string(),
        columns: vec!["id"],
    };
    let _qb = QB::<()>::new(base).select::<()>(vec![]);
}

#[test]
#[should_panic(
    expected = "Cannot create IN condition with empty value list. At least one value must be specified."
)]
fn in_empty_values_panics() {
    let col = Column::<i32> {
        name: "id",
        table_alias: "u",
        aliased_name: "u__id",
        _marker: PhantomData,
    };
    let _cond = col.in_(vec![]);
}

#[test]
#[should_panic(
    expected = "Cannot create NOT IN condition with empty value list. At least one value must be specified."
)]
fn not_in_empty_values_panics() {
    let col = Column::<i32> {
        name: "id",
        table_alias: "u",
        aliased_name: "u__id",
        _marker: PhantomData,
    };
    let _cond = col.not_in(vec![]);
}
