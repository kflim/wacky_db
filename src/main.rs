use std::io::{stdin, stdout, Write};

use rand::Rng;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sqlparser::{
    ast::{Assignment, ColumnOption, Ident, ObjectName, Statement},
    dialect::GenericDialect,
    parser::Parser,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Record {
    id: u32,
    data: String,
}

struct WackyDB {
    conn: Connection,
}

pub struct ColumnDefinition {
    name: String,
    column_type: String,
    options: Vec<String>,
}

enum ChaosResult {
    GamingTime,
    DatabaseOnFire,
    DataInTrash,
    NothingHappened,
    ProceedAsNormal,
}

impl WackyDB {
    pub fn new(db_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let sanitized_name = sanitize_db_name(db_name)?;

        // Open a database connection
        let conn = Connection::open(&sanitized_name)?;
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS records (
                    id INTEGER PRIMARY KEY
                    data TEXT NOT NULL
        )",
            [],
        );

        Ok(Self { conn })
    }

    fn chaos_engine() -> ChaosResult {
        let mut rng = rand::thread_rng();
        if rng.gen_range(0..10) < 3 {
            println!("Something WaCky is hApennning!");
            let roll_outcome = rng.gen_range(0..5);
            match roll_outcome {
                0 => {
                    return ChaosResult::GamingTime;
                }
                1 => {
                    return ChaosResult::DatabaseOnFire;
                }
                2 => {
                    return ChaosResult::DataInTrash;
                }
                _ => {
                    return ChaosResult::NothingHappened;
                }
            }
        }

        ChaosResult::ProceedAsNormal
    }

    fn table_exists(&self, table_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";

        let mut stmt = self.conn.prepare(query)?;
        let mut rows = stmt.query([table_name])?;

        // If the query returns a row, the table exists
        Ok(rows.next()?.is_some())
    }

    pub fn create_table(
        &self,
        table_name: &str,
        columns: &[ColumnDefinition], // Use the new structure
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.table_exists(table_name)? {
            return Err(format!("Table '{}' already exists.", table_name).into());
        }

        let chaos_result = Self::chaos_engine();

        match chaos_result {
            ChaosResult::GamingTime => {
                println!("It's time to play a classic!");
                let outcome = play_game();
                if outcome {
                    return Ok(());
                } else {
                    return Err(
                        "You lost the game! You also lost some PRECIOUS data as well!".into(),
                    );
                }
            }
            ChaosResult::DatabaseOnFire => {
                return Err("Oh no, the database is on fire! 🔥".into());
            }
            ChaosResult::DataInTrash => {
                return Err(
                    "Oops, I dropped your data in the trash! I think I can recover it?".into(),
                );
            }
            ChaosResult::NothingHappened => {
                return Err("Nah, nothing happened. I'm feelin a little QUIRKY today".into());
            }
            _ => {
                // Construct the SQL CREATE TABLE statement
                let columns_definition: Vec<String> = columns
                    .iter()
                    .map(|col| {
                        let mut definition = format!("{} {}", col.name, col.column_type);
                        for option in &col.options {
                            definition.push_str(&format!(" {}", option));
                        }
                        definition
                    })
                    .collect();
                let columns_str = columns_definition.join(", ");

                let sql = format!("CREATE TABLE {} ({})", table_name, columns_str);

                // Prepare and execute the statement
                let mut stmt = self.conn.prepare(&sql)?;
                stmt.execute([])?; // No parameters to bind since the SQL is already constructed.
            }
        }

        Ok(())
    }

    pub fn insert(
        &self,
        table_name: &str,
        values: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let chaos_result = Self::chaos_engine();

        match chaos_result {
            ChaosResult::GamingTime => {
                println!("It's time to play a classic!");
                let outcome = play_game();
                if outcome {
                    return Ok(());
                } else {
                    return Err(
                        "You lost the game! You also lost some PRECIOUS data as well!".into(),
                    );
                }
            }
            ChaosResult::DatabaseOnFire => {
                return Err("Oh no, the database is on fire! 🔥".into());
            }
            ChaosResult::DataInTrash => {
                return Err(
                    "Oops, I dropped your data in the trash! I think I can recover it?".into(),
                );
            }
            ChaosResult::NothingHappened => {
                return Err("Nah, nothing happened. I'm feelin a little QUIRKY today".into());
            }
            _ => {
                // Prepare the SQL statement
                let placeholders = values.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
                let sql = format!("INSERT INTO {} VALUES ({})", table_name, placeholders);
                let mut stmt = self.conn.prepare(&sql)?;

                // Execute the statement with the values
                let params: Vec<&dyn rusqlite::ToSql> =
                    values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
                stmt.execute(params.as_slice())?;
            }
        }

        Ok(())
    }

    fn select(&self, table_name: &str, columns: &str, where_clauses: &Vec<Assignment>) {
        if !self.table_exists(table_name).unwrap_or(false) {
            println!("Error: Table '{}' does not exist.", table_name);
            return;
        }

        let mut where_str = String::new();
        for assignment in where_clauses {
            match &assignment.value {
                sqlparser::ast::Expr::Value(value) => match value {
                    sqlparser::ast::Value::SingleQuotedString(s) => {
                        where_str.push_str(&format!("{} = '{}', ", assignment.target, s));
                    }
                    sqlparser::ast::Value::Number(n, _) => {
                        where_str.push_str(&format!("{} = {}, ", assignment.target, n));
                    }
                    _ => {
                        println!("Unimplemented value:\n {:?}", value);
                    }
                },
                _ => {
                    println!("Unimplemented expression:\n {:?}", assignment.value);
                }
            }
        }

        // Remove the trailing comma and space
        where_str.pop();
        where_str.pop();

        if where_str.is_empty() {
            where_str = "1 = 1".to_string();
        }

        let sql = format!("SELECT {} FROM {} WHERE {}", columns, table_name, where_str);
        let mut stmt = self.conn.prepare(&sql).unwrap();
        let mut rows = stmt.query([]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            println!("{:?}", row);
        }
    }

    fn update(
        &self,
        table_name: &str,
        assignments: &Vec<Assignment>,
        where_clauses: &Vec<Assignment>,
    ) {
        let mut set_str = String::new();
        for assignment in assignments {
            match &assignment.value {
                sqlparser::ast::Expr::Value(value) => match value {
                    sqlparser::ast::Value::SingleQuotedString(s) => {
                        set_str.push_str(&format!("{} = '{}', ", assignment.target, s));
                    }
                    sqlparser::ast::Value::Number(n, _) => {
                        set_str.push_str(&format!("{} = {}, ", assignment.target, n));
                    }
                    _ => {
                        println!("Unimplemented value:\n {:?}", value);
                    }
                },
                _ => {
                    println!("Unimplemented expression:\n {:?}", assignment.value);
                }
            }
        }

        // Remove the trailing comma and space
        set_str.pop();
        set_str.pop();

        let mut where_str = String::new();
        for assignment in where_clauses {
            match &assignment.value {
                sqlparser::ast::Expr::Value(value) => match value {
                    sqlparser::ast::Value::SingleQuotedString(s) => {
                        where_str.push_str(&format!("{} = '{}', ", assignment.target, s));
                    }
                    sqlparser::ast::Value::Number(n, _) => {
                        where_str.push_str(&format!("{} = {}, ", assignment.target, n));
                    }
                    _ => {
                        println!("Unimplemented value:\n {:?}", value);
                    }
                },
                _ => {
                    println!("Unimplemented expression:\n {:?}", assignment.value);
                }
            }
        }

        // Remove the trailing comma and space
        where_str.pop();
        where_str.pop();

        let sql = format!("UPDATE {} SET {} WHERE {}", table_name, set_str, where_str);
        let mut stmt = self.conn.prepare(&sql).unwrap();
        stmt.execute([]).unwrap();
    }
}

fn sanitize_db_name(name: &str) -> Result<&str, &str> {
    if name.is_empty() {
        return Err("Database name cannot be empty, even for a wacky database!");
    }

    let is_valid = name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.');
    if !is_valid {
        return Err(
            "Database name can only contain alphanumeric characters, underscores and dots, ごめん!",
        );
    }

    Ok(name)
}

fn play_game() -> bool {
    return true;
}

fn main() {
    let db = WackyDB::new("wacky_db.sqlite").unwrap();
    let sql_dialect = GenericDialect {};

    println!("Welcome to WackyDB, the wackiest database you will see! (Today at least)");
    loop {
        print!("wacky_db> ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        if input.trim().eq_ignore_ascii_case("quit") {
            println!("Goodbye! Thanks for using WackyDB!");
            break;
        }

        match Parser::parse_sql(&sql_dialect, &input) {
            Ok(statements) => {
                for statement in statements {
                    match statement {
                        // Update to handle more options later
                        Statement::CreateTable(create_table) => {
                            let object_name = create_table.name.0;
                            if object_name.len() > 1 {
                                println!("Sorry, WackyDB doesn't do schemas or databases! Only tables are allowed in this land.");
                                continue;
                            }
                            let table_name = object_name[0].value.clone(); // Get the table name

                            let columns_definition: Vec<ColumnDefinition> = create_table
                                .columns
                                .iter()
                                .map(|column| {
                                    let column_type = match &column.data_type {
                                        sqlparser::ast::DataType::Integer(_) => {
                                            "INTEGER".to_string()
                                        }
                                        sqlparser::ast::DataType::Text => "TEXT".to_string(),
                                        _ => "UNKNOWN".to_string(), // Handle other types as necessary
                                    };

                                    let options = column
                                        .options
                                        .iter()
                                        .map(|opt| match opt.option {
                                            ColumnOption::NotNull => "NOT NULL".to_string(),
                                            ColumnOption::Unique {
                                                is_primary,
                                                characteristics, // Bahahaha I'll do this later
                                            } => {
                                                if is_primary {
                                                    "PRIMARY KEY".to_string()
                                                } else {
                                                    "UNIQUE".to_string()
                                                }
                                            }
                                            _ => String::new(),
                                        })
                                        .filter(|opt| !opt.is_empty()) // Filter out empty options
                                        .collect();

                                    ColumnDefinition {
                                        name: column.name.value.clone(),
                                        column_type,
                                        options,
                                    }
                                })
                                .collect();

                            // Call create_table with the extracted information
                            if let Err(e) = db.create_table(&table_name, &columns_definition) {
                                println!("Error creating table: {}", e);
                            }
                        }
                        // Update to handle more options later
                        Statement::Insert(insert) => {
                            let object_name = insert.table_name.0;

                            // Check if the table exists
                            if object_name.len() > 1 {
                                println!("Sorry, WackyDB doesn't do schemas or databases! Only tables are allowed in this land.");
                                continue;
                            }

                            let table_name = object_name[0].clone();

                            // Check if the table exists
                            if !db.table_exists(&table_name.value).unwrap_or(false) {
                                println!("Error: Table '{}' does not exist.", table_name.value);
                                continue;
                            }

                            let data = insert.source;
                            if let Some(data) = data {
                                let body = data.body;

                                match *body {
                                    sqlparser::ast::SetExpr::Values(values) => {
                                        let instances = values.rows;

                                        for instance in instances {
                                            let mut insert_values: Vec<String> = Vec::new(); // or Vec<sqlparser::ast::Value> if you want to keep the original types

                                            for value in instance {
                                                match value {
                                                    sqlparser::ast::Expr::Value(value) => {
                                                        match value {
                                                            sqlparser::ast::Value::SingleQuotedString(s) => {
                                                                insert_values.push(s.clone());
                                                            }
                                                            sqlparser::ast::Value::Number(n, _) => {
                                                                insert_values.push(n.clone());
                                                            }
                                                            // Add other cases if needed
                                                            _ => {
                                                                println!("Unimplemented value:\n {:?}", value);
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        println!(
                                                            "Unimplemented expression:\n {:?}",
                                                            value
                                                        );
                                                    }
                                                }
                                            }

                                            // Now just insert the collected values into the database
                                            if let Err(e) =
                                                db.insert(&table_name.value, insert_values)
                                            {
                                                println!("Insert error: {}", e);
                                            }
                                        }
                                    }
                                    _ => {
                                        println!("Unimplemented body:\n {:?}", body);
                                    }
                                }
                            }
                        }
                        // Update to handle more options later
                        Statement::Query(query) => {
                            let body = query.body;
                            if let sqlparser::ast::SetExpr::Select(select) = *body {
                                let projection = select.projection;
                                let from = select.from;
                                let selection = select.selection;

                                if !from.is_empty() {
                                    let table_name = match &from[0].relation {
                                        sqlparser::ast::TableFactor::Table {
                                            name, alias, ..
                                        } => {
                                            let object_name = &name.0;
                                            if object_name.len() > 1 {
                                                println!("Sorry, WackyDB doesn't do schemas or databases! Only tables are allowed in this land.");
                                                continue;
                                            }
                                            object_name[0].clone().value
                                        }
                                        _ => {
                                            println!("Unimplemented table:\n {:?}", from[0]);
                                            continue;
                                        }
                                    };

                                    // Check if the table exists
                                    if !db.table_exists(&table_name).unwrap_or(false) {
                                        println!("Error: Table '{}' does not exist.", table_name);
                                        continue;
                                    }

                                    let columns = projection
                                        .iter()
                                        .map(|col| col.to_string())
                                        .collect::<Vec<String>>()
                                        .join(", ");

                                    let mut where_clauses = Vec::new();
                                    if let Some(ref selection) = selection {
                                        match selection {
                                            sqlparser::ast::Expr::BinaryOp { left, op, right } => {
                                                let column = match **left {
                                                    sqlparser::ast::Expr::Identifier(ref id) => {
                                                        let column = id.value.clone();
                                                        column
                                                    }
                                                    _ => {
                                                        println!(
                                                            "Unimplemented selection:\n {:?}",
                                                            selection
                                                        );
                                                        continue;
                                                    }
                                                };
                                                let value = match **right {
                                                    sqlparser::ast::Expr::Value(ref value) => match value {
                                                        sqlparser::ast::Value::SingleQuotedString(s) => {
                                                            s.clone()
                                                        }
                                                        sqlparser::ast::Value::Number(n, _) => n.clone(),
                                                        _ => {
                                                            println!("Unimplemented value:\n {:?}", value);
                                                            continue;
                                                        }
                                                    },
                                                    _ => {
                                                        println!(
                                                            "Unimplemented selection:\n {:?}",
                                                            selection
                                                        );
                                                        continue;
                                                    }
                                                };
                                                where_clauses.push(Assignment {
                                                    target: sqlparser::ast::AssignmentTarget::ColumnName(
                                                        ObjectName {
                                                            0: vec![Ident {
                                                                value: column,
                                                                quote_style: None,
                                                            }],
                                                        },
                                                    ),
                                                    value: sqlparser::ast::Expr::Value(
                                                        sqlparser::ast::Value::SingleQuotedString(value),
                                                    ),
                                                });
                                            }
                                            _ => {
                                                println!(
                                                    "Unimplemented selection:\n {:?}",
                                                    selection
                                                );
                                            }
                                        }
                                    }
                                    db.select(&table_name, &columns, &where_clauses);
                                }
                            }
                        }
                        // Update to handle more options later
                        Statement::Update {
                            table,
                            assignments,
                            from,
                            selection,
                            returning,
                        } => {
                            let table_name = match table.relation {
                                sqlparser::ast::TableFactor::Table { name, alias, .. } => {
                                    let object_name = name.0;
                                    if object_name.len() > 1 {
                                        println!("Sorry, WackyDB doesn't do schemas or databases! Only tables are allowed in this land.");
                                        continue;
                                    }
                                    object_name[0].clone().value
                                }
                                _ => {
                                    println!("Unimplemented table:\n {:?}", table);
                                    continue;
                                }
                            };

                            // Check if the table exists
                            if !db.table_exists(&table_name).unwrap_or(false) {
                                println!("Error: Table '{}' does not exist.", table_name);
                                continue;
                            }

                            let mut where_clauses = Vec::new();
                            if let Some(ref selection) = selection {
                                match selection {
                                    sqlparser::ast::Expr::BinaryOp { left, op, right } => {
                                        let column = match **left {
                                            sqlparser::ast::Expr::Identifier(ref id) => {
                                                let column = id.value.clone();
                                                column
                                            }
                                            _ => {
                                                println!(
                                                    "Unimplemented selection:\n {:?}",
                                                    selection
                                                );
                                                continue;
                                            }
                                        };
                                        let value = match **right {
                                            sqlparser::ast::Expr::Value(ref value) => match value {
                                                sqlparser::ast::Value::SingleQuotedString(s) => {
                                                    s.clone()
                                                }
                                                sqlparser::ast::Value::Number(n, _) => n.clone(),
                                                _ => {
                                                    println!("Unimplemented value:\n {:?}", value);
                                                    continue;
                                                }
                                            },
                                            _ => {
                                                println!(
                                                    "Unimplemented selection:\n {:?}",
                                                    selection
                                                );
                                                continue;
                                            }
                                        };
                                        where_clauses.push(Assignment {
                                            target: sqlparser::ast::AssignmentTarget::ColumnName(
                                                ObjectName {
                                                    0: vec![Ident {
                                                        value: column,
                                                        quote_style: None,
                                                    }],
                                                },
                                            ),
                                            value: sqlparser::ast::Expr::Value(
                                                sqlparser::ast::Value::SingleQuotedString(value),
                                            ),
                                        });
                                    }
                                    _ => {
                                        println!("Unimplemented selection:\n {:?}", selection);
                                    }
                                }
                            }
                            db.update(&table_name, &assignments, &where_clauses);
                        }
                        // Update to handle more options later
                        Statement::Drop {
                            object_type, names, ..
                        } => {
                            println!("{} {:?}", object_type, names);
                            for name in names {
                                let object_name = &name.0;
                                if object_name.len() > 1 {
                                    println!("Sorry, WackyDB doesn't do schemas or databases! Only tables are allowed in this land.");
                                    return;
                                }
                                let table_name = object_name[0].clone().value;

                                if let Err(e) =
                                    db.conn.execute(&format!("DROP TABLE {}", table_name), [])
                                {
                                    println!("{}", e);
                                }
                            }
                        }
                        _ => {
                            println!("Unimplemented statement:\n {:?}", statement);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Parsing error huh. This is what happened:\n{}", e);
            }
        }
    }
}
