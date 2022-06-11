//! This is a niche library of convenience types, structs, and
//! functions for projects with sqlx, postgres, async_graphql.

extern crate async_graphql;
use async_graphql::{Context, FieldError, FieldResult};
use async_trait::async_trait;
use deadpool::managed::{Manager, Object, RecycleResult};
use sqlx::{Connection, Error as SqlxError, PgConnection};

/// A replacement for sqlx's connection pool using deadpool
pub type Pool = deadpool::managed::Pool<PoolManager>;

/// A pool manager for managing sqlx database connections
pub struct PoolManager {
    pub url: String,
}

#[async_trait]
/// Example usage
/// ```ignore
///  // somewhere in your initialization code for your graphql server
///  let mgr = PoolManager { url: database_url };
///  let db_pool = Pool::new(mgr, 16);
///  async_graphql::Schema::build(QueryRoot::default(),
///     EmptyMutation::default(), EmptySubscription).data(pool);
/// ```
impl Manager for PoolManager {
    type Type = PgConnection;
    type Error = SqlxError;
    async fn create(&self) -> Result<PgConnection, SqlxError> {
        PgConnection::connect(&self.url).await
    }
    async fn recycle(&self, obj: &mut PgConnection) -> RecycleResult<SqlxError> {
        Ok(obj.ping().await?)
    }

    fn detach(&self, _obj: &mut Self::Type) {}
}

/// This is a convenience function that performs match on a Result type and if
/// is an error, prepends a helpful error message to the Err returned.
///
/// Returns Ok or Err with a custom error message
/// # Arguments
/// * `res` - a Result type to evaluate
/// * `err_msg` - a custom error message that will be prepended if Err is returned  
pub fn match_result<T>(res: Result<T, SqlxError>, err_msg: String) -> FieldResult<T> {
    match res {
        Ok(res) => Ok(res),
        Err(e) => {
            return Err(FieldError {
                message: format!("{} {:?}", err_msg, e),
                extensions: None,
                source: None,
            });
        }
    }
}

/// Extracts a connection object out of the Pool. Caller will still need to call
/// a deref_mut() on the returned object to get the object dereferenced in its
/// correct Type. This function assumes you have graphQL context that has a Pool
/// object defined in it.
///
/// Returns a Result with the connection object or a graphQL error
/// # Arguments
/// * `ctx` - graphQL context where the Pool object is stored
///
/// Example usage
/// ```ignore
/// // somewhere in your resolver code path on your graphql server
/// let mut db_conn = get_db_connection(ctx).await?;
//  let query_str = format!(
///       r#"
///         SELECT * FROM my_data
///       "#,
///     );
///     let row = query_as::<_, MyData>(query_str.as_str())
///       .fetch_all(db_conn.deref_mut())
///       .await;
///     match_result(
///       row,
///       format!("Failed to get my_data"),
///     )
/// ```
pub async fn get_db_connection(
    ctx: &Context<'_>,
) -> Result<Object<PoolManager>, async_graphql::FieldError> {
    let pool = ctx.data::<Pool>().unwrap(); // this cannot fail - panic if failure
    pool.get().await.map_err(|e| async_graphql::FieldError {
        message: format!("Database connectivity error: {:?}", e.to_string()),
        extensions: None,
        source: None,
    })
}
