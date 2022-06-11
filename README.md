# sqlx_helpers

library of helper functions for using sqlx with deadpool and async_graphql

More specifically, as of version 0.5.12 of sqlx, the default pool provided by the sqlx crate is somewhat buggy.
See https://github.com/launchbadge/sqlx/issues/622. Even though the issue is marked as closed, we are still seeing the problem described there.
In this library, we added a Pool replacement along with a a convenience function for fetching a database connection from the pool.
