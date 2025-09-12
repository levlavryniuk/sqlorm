pub fn format_alised_col_name(alias: &str, column_name: &str) -> String {
    format!("{}__{}", alias, column_name)
}
