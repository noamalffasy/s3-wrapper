use std::fmt::Write;

use sea_orm::{
    sea_query::{Func, FunctionCall, SimpleExpr},
    Iden,
};

struct Substr;

impl Iden for Substr {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "SUBSTR").unwrap();
    }
}

/// Call `Substr` function.
///
/// # Examples
///
/// ```
/// use sea_query::{tests_cfg::*, *};
///
/// let query = Query::select()
///     .expr(substr(Expr::col(Char::Character), Expr::expr(1), Expr::expr(6)))
///     .from(Char::Table)
///     .to_owned();
///
/// assert_eq!(
///     query.to_string(MysqlQueryBuilder),
///     r#"SELECT SUBSTR(`character`, 1, 6) FROM `character`"#
/// );
/// assert_eq!(
///     query.to_string(PostgresQueryBuilder),
///     r#"SELECT SUBSTR("character", 1, 6) FROM "character""#
/// );
/// assert_eq!(
///     query.to_string(SqliteQueryBuilder),
///     r#"SELECT SUBSTR("character", 1, 6) FROM "character""#
/// );
/// ```
pub fn substr<A, B, C>(str: A, start: B, length: C) -> FunctionCall
where
    A: Into<SimpleExpr>,
    B: Into<SimpleExpr>,
    C: Into<SimpleExpr>,
{
    Func::cust(Substr).args([str.into(), start.into(), length.into()])
}

#[cfg(test)]
mod tests {
    use sea_query::{tests_cfg::*, *};

    use super::substr;

    fn build_query() -> SelectStatement {
        return Query::select()
            .expr(substr(
                Expr::col(Char::Character),
                Expr::expr(1),
                Expr::expr(6),
            ))
            .from(Char::Table)
            .to_owned();
    }

    #[test]
    fn test_function_calling_in_mysql() {
        let query = build_query();

        assert_eq!(
            query.to_string(MysqlQueryBuilder),
            r#"SELECT SUBSTR(`character`, 1, 6) FROM `character`"#
        );
    }
    #[test]
    fn test_function_calling_in_postgres() {
        let query = build_query();

        assert_eq!(
            query.to_string(PostgresQueryBuilder),
            r#"SELECT SUBSTR("character", 1, 6) FROM "character""#
        );
    }
    #[test]
    fn test_function_calling_in_sqlite() {
        let query = build_query();

        assert_eq!(
            query.to_string(SqliteQueryBuilder),
            r#"SELECT SUBSTR("character", 1, 6) FROM "character""#
        );
    }
}
