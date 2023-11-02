use std::fmt::Write;

use sea_orm::{
    sea_query::{Func, FunctionCall, SimpleExpr},
    Iden,
};

struct Instr;

impl Iden for Instr {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "INSTR").unwrap();
    }
}

/// Call `Instr` function.
///
/// # Examples
///
/// ```
/// use sea_query::{tests_cfg::*, *};
///
/// let query = Query::select()
///     .expr(instr(Expr::col(Char::Character), Expr::expr(',')))
///     .from(Char::Table)
///     .to_owned();
///
/// assert_eq!(
///     query.to_string(MysqlQueryBuilder),
///     r#"SELECT INSTR(`character`, ',') FROM `character`"#
/// );
/// assert_eq!(
///     query.to_string(PostgresQueryBuilder),
///     r#"SELECT INSTR("character", ',') FROM "character""#
/// );
/// assert_eq!(
///     query.to_string(SqliteQueryBuilder),
///     r#"SELECT INSTR("character", ',') FROM "character""#
/// );
/// ```
pub fn instr<A, B>(str: A, substr: B) -> FunctionCall
where
    A: Into<SimpleExpr>,
    B: Into<SimpleExpr>,
{
    Func::cust(Instr).args([str.into(), substr.into()])
}
