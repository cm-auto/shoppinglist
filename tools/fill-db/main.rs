macro_rules! path_relative_to_crate_root {
    ($path: literal) => {
        concat!("../../", $path)
    };
}

#[actix_web::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");

    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pg_pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // check if username 'alice' already exists
    let record_option = sqlx::query!("select id from users where username = 'alice'")
        .fetch_optional(&pg_pool)
        .await
        .unwrap();
    if record_option.is_none() {
        let insert_users = include_str!(path_relative_to_crate_root!("db-filler-files/users.sql"));
        sqlx::query(insert_users).execute(&pg_pool).await.unwrap();
        println!("Inserted users");

        let insert_groups =
            include_str!(path_relative_to_crate_root!("db-filler-files/groups.sql"));
        sqlx::query(insert_groups).execute(&pg_pool).await.unwrap();
        println!("Inserted groups");

        let insert_users_groups_relations = include_str!(path_relative_to_crate_root!(
            "db-filler-files/users_groups_relations.sql"
        ));
        sqlx::query(insert_users_groups_relations)
            .execute(&pg_pool)
            .await
            .unwrap();
        println!("Inserted users_groups_relations");

        let insert_entries =
            include_str!(path_relative_to_crate_root!("db-filler-files/entries.sql"));
        sqlx::query(insert_entries).execute(&pg_pool).await.unwrap();
        println!("Inserted entries");
    }
}
