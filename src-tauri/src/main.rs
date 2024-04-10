// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

use thiserror::Error;
use xlsxwriter::*;



#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
use rusqlite::{params, Connection, Result};


#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    // Add more error types as needed
}

impl From<AppError> for tauri::InvokeError {
    fn from(error: AppError) -> Self {
        // You can customize the error message or conversion logic as needed.
        // This example simply converts the error to a string message.
        tauri::InvokeError::from(format!("{}", error))
    }
}


#[derive(Debug, Clone,serde::Serialize,serde::Deserialize)]
pub struct Spec {
    quantity: f32,
    orig_price: f64,
    sale_price: f64,
    liquidation_date: String,
}
#[derive(Debug, Clone,serde::Serialize,serde::Deserialize)]
pub struct DateTime {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

#[derive(Debug, Clone,serde::Serialize,serde::Deserialize)]
pub enum TransactionType {
    Remove,
    Edit,
    Add,

}

impl DateTime {
    fn to_string(&self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }

    fn from_string(date_str: &str) -> Result<DateTime, &'static str> {
        let parts: Vec<&str> = date_str.split(|c: char| c == '-' || c == ' ' || c == ':').collect();
        if parts.len() != 6 {
            return Err("Invalid date time format");
        }
        Ok(DateTime {
            year: parts[0].parse().map_err(|_| "Invalid year")?,
            month: parts[1].parse().map_err(|_| "Invalid month")?,
            day: parts[2].parse().map_err(|_| "Invalid day")?,
            hour: parts[3].parse().map_err(|_| "Invalid hour")?,
            minute: parts[4].parse().map_err(|_| "Invalid minute")?,
            second: parts[5].parse().map_err(|_| "Invalid second")?,
        })
    }
}


pub fn connect_and_setup_db() -> Result<Connection> {
    let conn = Connection::open("/Users/j-supha/desktop/Tao_Inventory.db")?;
    conn.execute(
    "CREATE TABLE IF NOT EXISTS app_settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        usage_type INTEGER NOT NULL
    )",
    [],
    )?;
    // Attempt to insert the default setting without checking if the table is empty
    // This will insert the row if the table is empty, and do nothing if the row already exists
    conn.execute(
        "INSERT OR IGNORE INTO app_settings (id, usage_type) VALUES (1, 1)",
        [],
    )?;

    // Existing table creation for timber_purchases
    conn.execute(
        "CREATE TABLE IF NOT EXISTS timber_purchases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            quantity REAL NOT NULL,
            price_per_ton REAL NOT NULL,
            purchase_date TEXT NOT NULL,
            acquisition_value REAL AS (quantity * price_per_ton)
        )",
        [],
    )?;

    // New table for used timber
    conn.execute(
        "CREATE TABLE IF NOT EXISTS used_timber (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            quantity REAL NOT NULL,
            orig_price REAL NOT NULL,
            sell_price REAL NOT NULL,
            liquidation_date TEXT NOT NULL,
            orig_value REAL AS (quantity * orig_price),
            sell_value REAL AS (quantity * sell_price)
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS all_transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            quantity REAL NOT NULL,
            price_per_ton REAL,
            orig_price REAL,
            sell_price REAL,
            liquidation_date TEXT,
            purchase_date TEXT,
            is_used BOOLEAN NOT NULL
        )",
        [],
    )?;


    Ok(conn)
}

// Adjusted to accept date_time as a parameter in the format "DD-MM-YYYY HH:MM:SS"

#[tauri::command]
fn record_purchase(quantity: f32, price_per_ton: f64, date_time: DateTime) -> String {
    println!("Recording things here");
    let conn_result = connect_and_setup_db();
    let conn = match conn_result {
        Ok(conn) => conn,
        Err(e) => return format!("Error connecting to database: {}", e),
    };
    println!("Connected to database");
    let date_time_str = date_time.to_string();
    let execute_result = conn.execute(
        "INSERT INTO timber_purchases (quantity, price_per_ton, purchase_date) VALUES (?1, ?2, ?3)",
        params![quantity, price_per_ton, date_time_str],
    );
    println!("Executed query");
    match execute_result {
        Ok(_) => "Completed".to_string(),
        Err(e) => format!("Error executing database operation: {}", e),
    }
}

#[tauri::command]
fn add_transaction(quantity: f32, price_per_ton: f64, orig_price: f64, sell_price: f64, purchase_date: DateTime, liquidation_date: DateTime, is_used: bool) -> String {
    println!("Recording things here");
    let conn_result = connect_and_setup_db();
    let conn = match conn_result {
        Ok(conn) => conn,
        Err(e) => return format!("Error connecting to database: {}", e),
    };
    println!("Connected to database");
    let purchase_date_str = purchase_date.to_string();
    let liquidation_date_str = liquidation_date.to_string();
    let execute_result = conn.execute(
        "INSERT INTO all_transactions (quantity, price_per_ton, orig_price, sell_price, purchase_date, liquidation_date, is_used) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![quantity, price_per_ton, orig_price, sell_price, purchase_date_str, liquidation_date_str, is_used],
    );
    println!("Executed query");
    match execute_result {
        Ok(_) => "Completed".to_string(),
        Err(e) => format!("Error executing database operation: {}", e),
    }
}

#[tauri::command]
fn remove_transaction_via_id(id: i32) -> String {
    let conn_result = connect_and_setup_db();
    let conn = match conn_result {
        Ok(conn) => conn,
        Err(e) => return format!("Error connecting to database: {}", e),
    };
    let execute_result = conn.execute(
        "DELETE FROM all_transactions WHERE id = ?1",
        params![id],
    );
    match execute_result {
        Ok(_) => "Completed".to_string(),
        Err(e) => format!("Error executing database operation: {}", e),
    }
}

#[tauri::command]
fn edit_transaction_via_id(id: i32, quantity: f32, price_per_ton: f64, orig_price: f64, sell_price: f64, purchase_date: DateTime, liquidation_date: DateTime, is_used: bool) -> String {
    let conn_result = connect_and_setup_db();
    let conn = match conn_result {
        Ok(conn) => conn,
        Err(e) => return format!("Error connecting to database: {}", e),
    };
    let purchase_date_str = purchase_date.to_string();
    let liquidation_date_str = liquidation_date.to_string();
    let execute_result = conn.execute(
        "UPDATE all_transactions SET quantity = ?1, price_per_ton = ?2, orig_price = ?3, sell_price = ?4, purchase_date = ?5, liquidation_date = ?6, is_used = ?7 WHERE id = ?8",
        params![quantity, price_per_ton, orig_price, sell_price, purchase_date_str, liquidation_date_str, is_used, id],
    );
    match execute_result {
        Ok(_) => "Completed".to_string(),
        Err(e) => format!("Error executing database operation: {}", e),
    }
}

#[tauri::command]
fn check_inventory(quantity_needed: f32) -> Result<bool> {
    let conn = connect_and_setup_db()?;

    let total_quantity: f32 = conn.query_row(
        "SELECT SUM(quantity) FROM timber_purchases",
        [],
        |row| row.get(0),
    )?;

    Ok(total_quantity >= quantity_needed)
}
#[derive(serde::Serialize,serde::Deserialize)]
pub struct TaoPurchase {
    pub quantity: Option<f32>,
    pub orig_price: Option<f64>,
    pub selling_price: Option<f64>,
    pub purchase_date: Option<String>,
    pub liquidation_date: Option<String>,
    //pub value: Option<f64>,
}

#[derive(serde::Serialize,serde::Deserialize)]
pub struct Statistics {
    pub acquisition_value: f64,
    pub sell_value: f64,
    pub orig_value: f64,
}


#[tauri::command]
fn print_inventory() -> Result<Vec<TaoPurchase>, AppError> {
    let conn = connect_and_setup_db()?;
    let mut stmt = conn.prepare("SELECT id, ROUND(quantity, 2), price_per_ton, purchase_date FROM timber_purchases")?;
    let timber_iter = stmt.query_map([], |row| {
        Ok(TaoPurchase {
            quantity: row.get(1)?,
            orig_price: row.get(2)?,
            purchase_date: row.get(3)?,
            liquidation_date: None,
            selling_price: None,
        })
    })?;

    let mut items = Vec::new();
    for timber in timber_iter {
        let item = timber?; // Propagate the error upwards with `?` if an error occurs
        items.push(item);
    }

    Ok(items)
}
#[tauri::command]
fn print_inventory_used() -> Result<Vec<TaoPurchase>, AppError> {
    let conn = connect_and_setup_db()?;
    
    // Assuming 'id' is not needed for TaoPurchase struct and 'price_per_ton' maps to 'orig_price'.
    // 'total_price' is assumed to be 'selling_price'.
    let mut stmt = conn.prepare("SELECT quantity, orig_price, sell_price, liquidation_date FROM used_timber")?;
    let timber_iter = stmt.query_map([], |row| {
        Ok(TaoPurchase {
            quantity: Some(row.get(0)?),
            orig_price: Some(row.get(1)?),
            selling_price: Some(row.get(2)?), // Mapped from 'total_price'.
            purchase_date: None, // No purchase date in the used_timber table, set as None.
            liquidation_date: Some(row.get(3)?), // Directly mapped from 'liquidation_date'.
        })
    })?;

    let mut items = Vec::new();
    for timber in timber_iter {
        match timber {
            Ok(item) => items.push(item),
            Err(e) => return Err(AppError::DatabaseError(e)),
        }
    }

    Ok(items)
}


#[tauri::command]
fn inventory_statistics()  -> Result<Statistics, AppError> {
    let conn = connect_and_setup_db()?;


    let acquisition_value: f64 = conn.query_row(
        "SELECT COALESCE(SUM(acquisition_value),0) FROM timber_purchases",
        [],
        |row| row.get(0),
    )?;

    let orig_value: f64 = conn.query_row(
        "SELECT ROUND(COALESCE(SUM(orig_value),0), 2) FROM used_timber",
        [],
        |row| row.get(0),
    )?;

    
    
    let sell_value: f64 = conn.query_row(
        "SELECT ROUND(COALESCE(SUM(sell_value), 0), 2) FROM used_timber",
        [],
        |row| row.get(0),
    )?;

    Ok(Statistics {
        acquisition_value,
        sell_value,
        orig_value,
    })
}


#[tauri::command]
fn use_tao(quantity_needed: f32, liquidation_date_time: DateTime, selling_price : f64) -> Result<Vec<Spec>, AppError> {
    let conn = connect_and_setup_db()?;

    if check_inventory(quantity_needed) == Ok(false) {
        println!("Not enough timber in inventory.");
        return Ok(Vec::new());
    }
    let mut remaining_quantity = quantity_needed;
    let mut used_timber = Vec::new();
    let mut style: i64 = conn.query_row(
        "SELECT usage_type FROM app_settings WHERE id = 1", 
        [], 
        |row| row.get(0)
    )?;

    let mut stmt = match(style) {
        1 => conn.prepare("SELECT id, quantity, price_per_ton FROM timber_purchases ORDER BY purchase_date ASC")?, // FIFO Implementation
        2 => conn.prepare("SELECT id, quantity, price_per_ton FROM timber_purchases ORDER BY purchase_date DESC")?, // LIFO Implementation
        3 => conn.prepare("SELECT id, quantity, price_per_ton FROM timber_purchases ORDER BY price_per_ton ASC")?, // LOFO Implementation
        _ => conn.prepare("SELECT id, quantity, price_per_ton FROM timber_purchases ORDER BY price_per_ton DESC")?, // HIFO Implemenetation - can change to LIFO or FIFO accordingly
    };
    
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let id: i32 = row.get(0)?;
        let quantity: f32 = row.get(1)?;
        let orig_price: f64 = row.get(2)?;

        let used_quantity = if quantity <= remaining_quantity { quantity } else { remaining_quantity };
        let _total_price = used_quantity as f64 * orig_price;

        if used_quantity == quantity {
            // Use up the entire batch and delete it
            conn.execute("DELETE FROM timber_purchases WHERE id = ?", params![id])?;
        } else {
            // Partially use the batch and update the remaining quantity
            conn.execute("UPDATE timber_purchases SET quantity = ? WHERE id = ?", params![quantity - used_quantity, id])?;
        }

        // Record used timber
        let liquidation_date_str = liquidation_date_time.to_string(); // Convert DateTime to string

        conn.execute(
            "INSERT INTO used_timber (quantity, orig_price, sell_price, liquidation_date) VALUES (?1, ?2, ?3, ?4)",
            params![used_quantity, orig_price, selling_price, liquidation_date_str],
        )?;

        used_timber.push(Spec {
            quantity: used_quantity,
            orig_price: orig_price,
            sale_price: selling_price,
            liquidation_date: liquidation_date_str,

        });

        remaining_quantity -= used_quantity;
        if remaining_quantity <= 0.0 { break; }
    }

    if remaining_quantity > 0.0 {
        println!("Warning: Not enough timber in inventory. Missing {} tonnes.", remaining_quantity);
    }

    Ok(used_timber)
}

#[tauri::command]
fn write_inventory_to_excel() -> Result<(), AppError> {
    let conn = connect_and_setup_db()?;
    // Create a new workbook. Note that `Workbook::new` takes ownership of the path string
    // and returns a Workbook object that we will own and manage.
    let workbook = Workbook::new("/Users/j-supha/desktop/inventory_report.xlsx").unwrap(); // Or use '?' for error propagation

    // Now that `workbook` is of type `Workbook`, you can call `add_worksheet` on it
    let mut timber_sheet = workbook.add_worksheet(Some("Actual")).unwrap(); // Again, consider using '?' for real applications
    let mut used_timber_sheet = workbook.add_worksheet(Some("Used")).unwrap();

    // Write headers for both sheets by calling a helper function (not shown here)
    write_headers(&mut timber_sheet, &["ID", "Quantity", "Price", "Purchase Date"]);
    write_headers(&mut used_timber_sheet, &["ID", "Quantity", "Orig Price", "Selling Price", "Liquidation Date"]);

    // Query and write data to the "Actual" timber sheet
    write_timber_purchases(&conn, &mut timber_sheet)?;

    // Query and write data to the "Used" timber sheet
    write_used_timber(&conn, &mut used_timber_sheet)?;

    // Close the workbook. This is where the Excel file is actually written to disk.
    // Again, using `unwrap()` for simplicity, but error handling is recommended.
    workbook.close().unwrap();

    Ok(())
}

fn write_headers(sheet: &mut Worksheet, headers: &[&str]) {
    let _ = headers.iter().enumerate().try_for_each(|(index, &header)| {
        sheet.write_string(0, index as u16, header, None)
    });
}

fn write_timber_purchases(conn: &Connection, sheet: &mut Worksheet) -> Result<(), AppError> {
    let mut stmt = conn.prepare("SELECT id, quantity, price_per_ton, purchase_date FROM timber_purchases")?;
    let timber_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, f32>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut num = 0;
    for (row_num, timber) in timber_iter.enumerate() {
        let (id, quantity, price_per_ton, purchase_date) = timber?;
        let _ = sheet.write_number(row_num as u32 + 1, 0, id.into(), None);
        let _ =sheet.write_number(row_num as u32 + 1, 1, quantity.into(), None);
        let _ =sheet.write_number(row_num as u32 + 1, 2, price_per_ton, None);
        let _ =sheet.write_string(row_num as u32 + 1, 3, &purchase_date, None);
        num = row_num +1 ;
    }

    let _ =sheet.write_string(num as u32 + 3, 0, "Inventory Value", None);
    let res = inventory_statistics();
    let stats = match res {
        Ok(stats) => stats,
        Err(_) => return Err(AppError::DatabaseError(rusqlite::Error::QueryReturnedNoRows)),
    };
    let _ =sheet.write_number(num as u32 + 3, 1, stats.acquisition_value, None);




    Ok(())
}

fn write_used_timber(conn: &Connection, sheet: &mut Worksheet) -> Result<(), AppError> {
    let mut stmt = conn.prepare("SELECT id, quantity, orig_price, sell_price, liquidation_date FROM used_timber")?;
    let timber_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, f32>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    let mut num = 0;
    for (row_num, timber) in timber_iter.enumerate() {
        let (id, quantity, price_per_ton, total_price, liquidation_date) = timber?;
        let _ =sheet.write_number(row_num as u32 + 1, 0, id.into(), None);
        let _ =sheet.write_number(row_num as u32 + 1, 1, quantity.into(), None);
        let _ =sheet.write_number(row_num as u32 + 1, 2, price_per_ton, None);
        let _ =sheet.write_number(row_num as u32 + 1, 3, total_price, None);
        let _ =sheet.write_string(row_num as u32 + 1, 4, &liquidation_date, None);
        num = row_num +1 ;
    }
    let _ =sheet.write_string(num as u32 + 3, 0, "Inventory Orig Value", None);
    let res = inventory_statistics();
    let stats = match res {
        Ok(stats) => stats,
        Err(_) => return Err(AppError::DatabaseError(rusqlite::Error::QueryReturnedNoRows)),
    };
    let _ =sheet.write_number(num as u32 + 3, 2, stats.orig_value, None);
    let _ =sheet.write_string(num as u32 + 4, 0, "Inventory Liquation Value", None);
    let _ =sheet.write_number(num as u32 + 4, 2, stats.sell_value, None);
    
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, record_purchase, print_inventory, print_inventory_used, use_tao, write_inventory_to_excel, inventory_statistics])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}





