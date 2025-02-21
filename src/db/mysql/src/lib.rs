use std::sync::Arc;

use mysql_async::prelude::Queryable;
use mysql_async::prelude::ToValue;
use mysql_async::Opts;
use mysql_async::Pool;
use mysql_async::Row;
use mysql_common::constants::ColumnType::*;
use toasty_core::driver::Capability;
use toasty_core::driver::Operation;
use toasty_core::driver::Response;
use toasty_core::schema::db::Schema;
use toasty_core::schema::db::Table;
use toasty_core::stmt;
use toasty_core::Driver;
use toasty_core::Result;

#[derive(Debug)]
pub struct MySQL {
    /// The PostgreSQL client.
    pool: Pool,
}
impl MySQL {
    pub async fn connect(params: &str) -> Result<Self> {
        let pool = Pool::from_url(params)?;
        Ok(Self { pool })
    }

    pub async fn create_table(&self, schema: &Schema, table: &Table) -> Result<()> {
        println!("todobien");
        let mut params = Vec::new();
        let sql = stmt::sql::Statement::create_table(table)
            .serialize(schema, &mut params)
            .into_inner();

        assert!(
            params.is_empty(),
            "creating a table shouldn't involve any parameters"
        );

        let mut connection = self.pool.get_conn().await?;
        connection.exec_drop(sql, ()).await?;

        // NOTE: `params` is guaranteed to be empty based on the assertion above. If
        // that changes, `params.clear()` should be called here.
        for index in &table.indices {
            if index.primary_key {
                continue;
            }

            let sql = stmt::sql::Statement::create_index(index)
                .serialize(schema, &mut params)
                .into_inner();
            assert!(
                params.is_empty(),
                "creating an index shouldn't involve any parameters"
            );

            connection.exec_drop(&sql, ()).await?;
        }
        println!("todobien");

        Ok(())
    }
    pub async fn drop_table(&self, schema: &Schema, table: &Table) -> Result<()> {
        println!("WHY WHY");
        let mut params = Vec::new();
        let sql = stmt::sql::Statement::drop_table(table)
            .serialize(schema, &mut params)
            .into_inner();

        assert!(
            params.is_empty(),
            "dropping a table shouldn't involve any parameters"
        );

        let mut conn = self.pool.get_conn().await?;
        println!("WHY WHY CONECTION!!!");
        println!("{sql}");

        conn.exec_drop(&sql, ()).await?;

        Ok(())
    }
}
#[toasty_core::async_trait]
impl Driver for MySQL {
    fn capability(&self) -> &Capability {
        &Capability::Sql
    }

    async fn register_schema(&mut self, _schema: &Schema) -> Result<()> {
        Ok(())
    }

    async fn exec(&self, schema: &Arc<Schema>, op: Operation) -> Result<Response> {
        let sql = match op {
            Operation::Insert(stmt) => stmt,
            Operation::QuerySql(query) => query.stmt,
            op => todo!("{:#?}", op),
        };

        let width = sql.returning_len();

        let mut params = Vec::new();
        let sql_as_str = stmt::sql::Serializer::new(schema)
            .serialize_stmt(&sql, &mut params)
            .to_numbered_args()
            .into_inner();

        let mut conn = self.pool.get_conn().await?;

        let stmt = conn.prep(&sql_as_str).await?;

        //TODO maybe sync
        let args = params
            .iter()
            .map(|param| param.to_value())
            .collect::<Vec<mysql_async::Value>>();

        if width.is_none() {
            let count: Vec<Row> = conn
                .exec(&stmt, mysql_async::Params::Positional(args))
                .await?;
            return Ok(Response::from_count(count.len()));
        }
        let results: Vec<_> = conn
            // TODO: is there way to get an iterator back here instead of collecting the
            // results into a `Vec`?
            .exec(&stmt, &args)
            .await?
            .into_iter()
            .flat_map(|row: Row| {
                let mut results = Vec::new();

                for i in 0..row.len() {
                    let column = &row.columns()[i];
                    results.push(mysql_to_toasty(i, &row, column));
                }

                results
            })
            .collect();

        Ok(Response::from_value_stream(stmt::ValueStream::from_vec(
            results,
        )))
    }

    async fn reset_db(&self, schema: &Schema) -> Result<()> {
        for table in &schema.tables {
            self.drop_table(schema, table).await?;

            println!("HOASDASD ALE");
            self.create_table(schema, table).await?;
        }

        Ok(())
    }
}

fn mysql_to_toasty(i: usize, row: &Row, column: &mysql_async::Column) -> stmt::Value {
    match column.column_type() {
        MYSQL_TYPE_DECIMAL => todo!(),
        MYSQL_TYPE_TINY => todo!(),
        MYSQL_TYPE_SHORT => todo!(),
        MYSQL_TYPE_LONG => todo!(),
        MYSQL_TYPE_FLOAT => todo!(),
        MYSQL_TYPE_DOUBLE => todo!(),
        MYSQL_TYPE_NULL => stmt::Value::Null,
        MYSQL_TYPE_TIMESTAMP => todo!(),
        MYSQL_TYPE_LONGLONG => todo!(),
        MYSQL_TYPE_INT24 => todo!(),
        MYSQL_TYPE_DATE => todo!(),
        MYSQL_TYPE_TIME => todo!(),
        MYSQL_TYPE_DATETIME => todo!(),
        MYSQL_TYPE_YEAR => todo!(),
        MYSQL_TYPE_NEWDATE => todo!(),
        MYSQL_TYPE_VARCHAR => row
            .get(i)
            .map(stmt::Value::String)
            .unwrap_or(stmt::Value::Null),
        MYSQL_TYPE_BIT => todo!(),
        MYSQL_TYPE_TIMESTAMP2 => todo!(),
        MYSQL_TYPE_DATETIME2 => todo!(),
        MYSQL_TYPE_TIME2 => todo!(),
        MYSQL_TYPE_TYPED_ARRAY => todo!(),
        MYSQL_TYPE_VECTOR => todo!(),
        MYSQL_TYPE_UNKNOWN => todo!(),
        MYSQL_TYPE_JSON => todo!(),
        MYSQL_TYPE_NEWDECIMAL => todo!(),
        MYSQL_TYPE_ENUM => todo!(),
        MYSQL_TYPE_SET => todo!(),
        MYSQL_TYPE_TINY_BLOB => todo!(),
        MYSQL_TYPE_MEDIUM_BLOB => todo!(),
        MYSQL_TYPE_LONG_BLOB => todo!(),
        MYSQL_TYPE_BLOB => todo!(),
        MYSQL_TYPE_VAR_STRING => row
            .get(i)
            .map(stmt::Value::String)
            .unwrap_or(stmt::Value::Null),
        MYSQL_TYPE_STRING => todo!(),
        MYSQL_TYPE_GEOMETRY => todo!(),
    }
}
