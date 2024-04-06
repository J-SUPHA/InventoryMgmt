use rusqlite::{params, Connection, Result};


#[derive(Debug)]
pub struct Spec {
    quantity: i32,
    price_per_ton: f64,
    total_price: f64,
}
#[derive(Debug, Clone)]
pub struct DateTime {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

pub impl DateTime {
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
    let conn = Connection::open("timber_inventory.db")?;

    // Existing table creation for timber_purchases
    conn.execute(
        "CREATE TABLE IF NOT EXISTS timber_purchases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            quantity INTEGER NOT NULL,
            price_per_ton REAL NOT NULL,
            purchase_date TEXT NOT NULL
        )",
        [],
    )?;

    // New table for used timber
    conn.execute(
        "CREATE TABLE IF NOT EXISTS used_timber (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            quantity INTEGER NOT NULL,
            price_per_ton REAL NOT NULL,
            total_price REAL NOT NULL,
            liquidation_date TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS purchase_transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timber_purchase_id INTEGER NOT NULL,
            action TEXT NOT NULL, -- 'purchase' or 'reverse_purchase'
            transaction_date TEXT NOT NULL,
            FOREIGN KEY(timber_purchase_id) REFERENCES timber_purchases(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS use_transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            used_timber_id INTEGER NOT NULL,
            action TEXT NOT NULL, -- 'use' or 'reverse_use'
            transaction_date TEXT NOT NULL,
            FOREIGN KEY(used_timber_id) REFERENCES used_timber(id)
        )",
        [],
    )?;
    Ok(conn)
}

// Adjusted to accept date_time as a parameter in the format "DD-MM-YYYY HH:MM:SS"
pub fn record_purchase(quantity: i32, price_per_ton: f64, date_time: &DateTime) -> Result<()> {
    let conn = connect_and_setup_db();
    let date_time_str = date_time.to_string();
    conn.execute(
        "INSERT INTO timber_purchases (quantity, price_per_ton, purchase_date) VALUES (?1, ?2, ?3)",
        params![quantity, price_per_ton, date_time_str],
    )?;
    Ok(())
}

pub fn check_inventory(quantity_needed: i32) -> Result<bool> {
    let conn = connect_and_setup_db();
    let total_quantity: i32 = conn.query_row(
        "SELECT SUM(quantity) FROM timber_purchases",
        [],
        |row| row.get(0),
    )?;
    Ok(total_quantity >= quantity_needed)
}

pub fn print_inventory() -> Result<()> {
    let conn = connect_and_setup_db();
    let mut stmt = conn.prepare("SELECT id, quantity, price_per_ton, purchase_date FROM timber_purchases")?;
    let timber_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, i32>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for timber in timber_iter {
        let (id, quantity, price_per_ton, purchase_date) = timber?;
        println!("ID: {}, Quantity: {}, Price/Ton: ${}, Purchase Date: {}", id, quantity, price_per_ton, purchase_date);
    }

    Ok(())
}

pub fn print_inventory_used() -> Result<()>{
    let conn = connect_and_setup_db();

    let mut stmt = conn.prepare("SELECT id, quantity, price_per_ton, total_price, liquidation_date FROM used_timber")?;
    let timber_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, i32>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    for used_timber in timber_iter {
        let (id, quantity, price_per_ton, total_price, liquidation_date) = used_timber?;
        println!("ID: {}, Quantity: {}, Price/Ton: ${}, Total Price: ${}, Liquidation Date: {}", id, quantity, price_per_ton, total_price, liquidation_date);
    }

    Ok(())
}

pub fn use_tao(quantity_needed: i32, liquidation_date_time: &DateTime) -> Result<Vec<Spec>> {
    let conn = connect_and_setup_db();

    if check_inventory(&conn, quantity_needed) == Ok(false) {
        println!("Not enough timber in inventory.");
        return Ok(Vec::new());
    }
    let mut remaining_quantity = quantity_needed;
    let mut used_timber = Vec::new();

    let mut stmt = conn.prepare("SELECT id, quantity, price_per_ton FROM timber_purchases ORDER BY id DESC")?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let id: i32 = row.get(0)?;
        let quantity: i32 = row.get(1)?;
        let price_per_ton: f64 = row.get(2)?;

        let used_quantity = if quantity <= remaining_quantity { quantity } else { remaining_quantity };
        let total_price = used_quantity as f64 * price_per_ton;

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
            "INSERT INTO used_timber (quantity, price_per_ton, total_price, liquidation_date) VALUES (?1, ?2, ?3, ?4)",
            params![used_quantity, price_per_ton, total_price, liquidation_date_str],
        )?;

        used_timber.push(Spec {
            quantity: used_quantity,
            price_per_ton,
            total_price,
        });

        remaining_quantity -= used_quantity;
        if remaining_quantity <= 0 { break; }
    }

    if remaining_quantity > 0 {
        println!("Warning: Not enough timber in inventory. Missing {} tonnes.", remaining_quantity);
    }

    Ok(used_timber)
}

