use std::{fmt::Display, path::Path};

use clap::{Parser, Subcommand};
use sqlx::{
    FromRow, Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteRow},
    types::chrono::{DateTime, Local},
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Day,
    Add { calories: u32, comment: String },
    Prev { days: u32 },
    Rm { id: u32 },
    All,
}

struct MealEntry {
    id: u32,
    name: String,
    calories: u32,
    created_at: DateTime<Local>,
}

impl Display for MealEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {}\n{} - {} kcal\non {}",
            self.id, self.name, self.calories, self.created_at
        )
    }
}

impl<'r> FromRow<'r, SqliteRow> for MealEntry {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            name: row.get("name"),
            calories: row.get("calories"),
            created_at: row.get("created_at"), // fails?
        })
    }
}

async fn query_days_ago(days: u32, pool: &SqlitePool) -> (Vec<MealEntry>, u32) {
    let meals = sqlx::query_as::<_, MealEntry>(
        "select * from meals where created_at >= datetime('now', '-1 days', 'start of day');",
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .expect("Failed to query meals");

    let total_calories = meals.iter().fold(0_u32, |mut acc, meal| {
        acc += meal.calories;
        acc
    });

    (meals, total_calories)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let home_dir = std::env::var("HOME").unwrap();
    let persistent_dir = Path::new(&home_dir).join(".calorie-counter/");

    if !persistent_dir.exists() {
        let _ = std::fs::create_dir(&persistent_dir);
    }

    let persistent = persistent_dir.join("calories.db");

    // create sqlite connections
    let pool = sqlx::SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(persistent.to_str().unwrap())
            .create_if_missing(true),
    )
    .await
    .expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS meals (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name VARCHAR(64) NOT NULL,
        calories INTEGER NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )",
    )
    .execute(&pool)
    .await
    .expect("Failed to initialise database");

    match cli.command {
        Commands::Day => {
            let (meals, calories) = query_days_ago(1, &pool).await;
            for meal in meals {
                println!("{meal}\n=======================")
            }
            println!("TOTAL CALORIES: {}", calories)
        }
        Commands::Add { calories, comment } => {
            sqlx::query("INSERT INTO meals (name, calories) VALUES (?1, ?2)")
                .bind(comment)
                .bind(calories)
                .execute(&pool)
                .await
                .expect("Failed to insert new meal");
            println!("Added new meal!")
        }
        Commands::Prev { days } => {
            let (meals, calories) = query_days_ago(days, &pool).await;
            for meal in meals {
                println!("{meal}\n=======================")
            }
            println!("TOTAL CALORIES: {}\nOver the last {} days", calories, days)
        }
        Commands::Rm { id } => {
            let meal = sqlx::query_as::<_, MealEntry>("SELECT * WHERE id = ?1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .expect("Failed to fetch query");

            if let Some(meal) = meal {
                // are you sure you want to delete
                let mut confirm = String::new();

                println!("{}\nAre you sure you want to delete? [y/n]\t", meal);
                let _ = std::io::stdin().read_line(&mut confirm);

                match confirm.to_lowercase().trim() {
                    "yes" | "y" => {
                        sqlx::query("DELETE FROM meals WHERE id = ?1")
                            .bind(id)
                            .execute(&pool)
                            .await
                            .expect("Failed to delete from database");
                        println!("deleted")
                    }
                    _ => {}
                }
            } else {
                println!("Not a valid index - doesn't exist")
            }
        }
        Commands::All => {
            let meals = sqlx::query_as::<_, MealEntry>("SELECT * FROM meals")
                .fetch_all(&pool)
                .await
                .expect("Failed to fetch");

            for meal in meals {
                println!("{meal}");
            }
        }
    }
}
