use crate::models::{Perimeter, Point, Region};
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

    pub async fn get_plots(&self) -> Result<Vec<Region>> {
        let mut rows =
            sqlx::query("SELECT plot_id, user_id, ax, az, bx, bz FROM plots").fetch(&self.pool);

        let mut res = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let region = Region {
                owner: row.try_get("user_id")?,
                name: row.try_get("plot_id")?,
                perimeter: Perimeter(
                    Point(row.try_get("ax")?, row.try_get("az")?),
                    Point(row.try_get("bx")?, row.try_get("bz")?),
                ),
            };
            res.push(region);
        }

        Ok(res)
    }

    pub async fn get_user_plots<I: Into<u64> + Copy>(&self, user_id: I) -> Result<Vec<Region>> {
        let mut rows = sqlx::query("SELECT plot_id, ax, az, bx, bz FROM plots WHERE user_id = ?")
            .bind(user_id.into())
            .fetch(&self.pool);

        let mut res = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let region = Region {
                owner: user_id.into(),
                name: row.try_get("plot_id")?,
                perimeter: Perimeter(
                    Point(row.try_get("ax")?, row.try_get("az")?),
                    Point(row.try_get("bx")?, row.try_get("bz")?),
                ),
            };
            res.push(region);
        }

        Ok(res)
    }

    pub async fn get_plot_by_name(&self, name: &str) -> Result<Option<Region>> {
        let mut rows = sqlx::query("SELECT user_id, ax, az, bx, bz FROM plots WHERE plot_id = ?")
            .bind(name)
            .fetch(&self.pool);

        if let Some(row) = rows.try_next().await? {
            let region = Region {
                name: name.to_owned(),
                owner: row.try_get("user_id")?,
                perimeter: Perimeter(
                    Point(row.try_get("ax")?, row.try_get("az")?),
                    Point(row.try_get("bx")?, row.try_get("bz")?),
                ),
            };
            Ok(Some(region))
        } else {
            Ok(None)
        }
    }

    pub async fn add_plot(&self, region: &Region) -> Result<()> {
        sqlx::query(
            "INSERT INTO plots (user_id, plot_id, ax, az, bx, bz) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(region.owner)
        .bind(&region.name)
        .bind(region.perimeter.0 .0)
        .bind(region.perimeter.0 .1)
        .bind(region.perimeter.1 .0)
        .bind(region.perimeter.1 .1)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_plot(&self, region: &Region) -> Result<()> {
        sqlx::query("UPDATE plots SET ax = ?, az = ?, bx = ?, bz = ? WHERE plot_id = ?")
            .bind(region.perimeter.0 .0)
            .bind(region.perimeter.0 .1)
            .bind(region.perimeter.1 .0)
            .bind(region.perimeter.1 .1)
            .bind(&region.name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_plot(&self, plot_name: &str) -> Result<()> {
        sqlx::query("DELETE FROM plots WHERE plot_id = ?")
            .bind(plot_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_plot_user_id<I: Into<u64> + Copy>(&self, user_id: I) -> Result<Option<i64>> {
        let mut rows = sqlx::query("SELECT plot_inc FROM plot_ids WHERE user_id = ?")
            .bind(user_id.into())
            .fetch(&self.pool);

        if let Some(row) = rows.try_next().await? {
            let inc = row.try_get("plot_inc")?;
            Ok(Some(inc))
        } else {
            Ok(None)
        }
    }

    pub async fn inc_plot_user_id<I: Into<u64> + Copy>(&self, id: I) -> Result<()> {
        let res = sqlx::query("UPDATE plot_ids SET plot_inc = plot_inc + 1 WHERE user_id = ?")
            .bind(id.into())
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            sqlx::query("INSERT INTO plot_ids (user_id, plot_inc) VALUES (?, ?)")
                .bind(id.into())
                .bind(1)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}
