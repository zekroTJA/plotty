pub mod models;

use anyhow::Result;
use serenity::futures::TryStreamExt;
use sqlx::{MySqlPool, Row};

pub struct Database {
    pool: MySqlPool,
}

impl Database {
    pub async fn new(dsn: &str) -> Result<Self> {
        let pool = MySqlPool::connect(dsn).await?;

        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;

        Ok(())
    }

    pub async fn get_user_by_id<I: Into<u64> + Copy>(&self, id: I) -> Result<Option<String>> {
        let mut rows = sqlx::query("SELECT mc_name FROM users WHERE user_id = ?")
            .bind(id.into())
            .fetch(&self.pool);

        if let Some(row) = rows.try_next().await? {
            let name: String = row.try_get("mc_name")?;
            Ok(Some(name))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_by_mcname(&self, mcname: &str) -> Result<Option<u64>> {
        let mut rows = sqlx::query("SELECT user_id FROM users WHERE mc_name = ?")
            .bind(mcname)
            .fetch(&self.pool);

        if let Some(row) = rows.try_next().await? {
            let id: u64 = row.try_get("user_id")?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    pub async fn set_user<I: Into<u64> + Copy>(&self, id: I, mcname: &str) -> Result<()> {
        let res = sqlx::query("UPDATE users SET mc_name = ? WHERE user_id = ?")
            .bind(mcname)
            .bind(id.into())
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            sqlx::query("INSERT INTO users (user_id, mc_name) VALUES (?, ?)")
                .bind(id.into())
                .bind(mcname)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn get_user_plots<I: Into<u64> + Copy>(&self, user_id: I) -> Result<Vec<String>> {
        let mut rows = sqlx::query("SELECT plot_id FROM plots WHERE user_id = ?")
            .bind(user_id.into())
            .fetch(&self.pool);

        let mut res = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id: String = row.try_get("plot_id")?;
            res.push(id);
        }

        Ok(res)
    }

    pub async fn get_plot_by_name(&self, name: &str) -> Result<Option<u64>> {
        let mut rows = sqlx::query("SELECT user_id FROM plots WHERE plot_id = ?")
            .bind(name)
            .fetch(&self.pool);

        if let Some(row) = rows.try_next().await? {
            let id: u64 = row.try_get("user_id")?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    pub async fn add_plot<I: Into<u64> + Copy>(&self, user_id: I, plot_name: &str) -> Result<()> {
        sqlx::query("INSERT INTO plots (user_id, plot_id) VALUES (?, ?)")
            .bind(user_id.into())
            .bind(plot_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
