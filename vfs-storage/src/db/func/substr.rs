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
