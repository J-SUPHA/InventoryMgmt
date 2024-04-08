import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

interface TaoPurchase {
  quantity: number | null;
  orig_price: number | null;
  selling_price: number | null;
  purchase_date: string | null;
  liquidation_date: string | null;
}

interface Statistics {
  acquisition_value: number;
  sell_value: number;
  orig_value: number;
}

interface Spec {
  quantity: number;
  orig_price: number;
  sale_price: number;
  liquidation_date: string;
}

function App() {
  const [inventory, setInventory] = useState<TaoPurchase[]>([]);
  const [usedInventory, setUsedInventory] = useState<TaoPurchase[]>([]);
  const [stats, setStats] = useState<Statistics>();
  const [quantityNeeded, setQuantityNeeded] = useState('');
  const [liquidationDateTime, setLiquidationDateTime] = useState('');
  const [quantity, setQuantity] = useState('');
  const [salePrice, setSalePrice] = useState('');
  const [price_per_ton, set_price_per_ton] = useState('');
  const [date, setDate] = useState('');

  useEffect(() => {
    fetchInventory();
    fetchUsedInventory();
    fetchStats();

  }, []);

  async function fetchInventory() {
    const inventory = await invoke("print_inventory") as TaoPurchase[];
    setInventory(inventory);
  }
  async function fetchStats() {
    const inventory = await invoke("inventory_statistics") as Statistics;
    console.log(inventory);
    setStats(inventory);
  }

  async function handleWriteToExcel() {
    await invoke("write_inventory_to_excel");
    alert("Inventory report generated successfully!");
  }

  async function fetchUsedInventory() {
    const inventoryUsed = await invoke("print_inventory_used") as TaoPurchase[];
    setUsedInventory(inventoryUsed);
  }

  async function handleRecordPurchase(e: React.FormEvent) {
    e.preventDefault();

    const dateTimeParts = date.split("-");
    let time_whole = dateTimeParts[2].split("T")[1];
    let time = time_whole.split(":");
    await invoke("record_purchase", { 
      quantity: parseInt(quantity), 
      pricePerTon: parseFloat(price_per_ton), 
      dateTime: {
        year: parseInt(dateTimeParts[0]), 
        month: parseInt(dateTimeParts[1]), 
        day: parseInt(dateTimeParts[2]), 
        hour: parseInt(time[0]), 
        minute: parseInt(time[1]), 
        second: 0 
      } 
    });
    fetchInventory();
    fetchUsedInventory();
  }

  async function handleUseTao(e: React.FormEvent) {
    e.preventDefault();
    const dateTimeParts = liquidationDateTime.split("-");
    console.log(dateTimeParts);
    let time_whole = dateTimeParts[2].split("T")[1];
    let time = time_whole.split(":");

    await invoke("use_tao", { 
      quantityNeeded: parseInt(quantityNeeded), 
      liquidationDateTime: {
        year: parseInt(dateTimeParts[0]), 
        month: parseInt(dateTimeParts[1]), 
        day: parseInt(dateTimeParts[2]), 
        hour: parseInt(time[0]), 
        minute: parseInt(time[1]), 
        second: 0 
      },
      sellingPrice: parseFloat(salePrice) 
    });
    fetchInventory();
    fetchUsedInventory();
  }

  return (
    <div className="App">
      <h1>Tauri Inventory Management</h1>
      <div className="row">
        <form onSubmit={handleRecordPurchase}>
          <div className="col">
            <input type="number" value={quantity} onChange={(e) => setQuantity(e.target.value)} placeholder="Quantity" />
            <input type="number" value={price_per_ton} onChange={(e) => set_price_per_ton(e.target.value)} placeholder="Price Per Tao" />
            <input type="datetime-local" value={date} onChange={(e) => setDate(e.target.value)} />
            <button type="submit">Record Purchase</button>
          </div>
        </form>
      
        <form onSubmit={handleUseTao}>
          <div className="col">
            <input type="number" value={quantityNeeded} onChange={(e) => setQuantityNeeded(e.target.value)} placeholder="Quantity Needed" />
            <input type="number" value={salePrice} onChange={(e) => setSalePrice(e.target.value)} placeholder="Selling Price" />
            <input type="datetime-local" value={liquidationDateTime} onChange={(e) => setLiquidationDateTime(e.target.value)} placeholder="Liquidation Date" />
            <button type="submit">Use Tao</button>
          </div>
        </form>
      </div>
      <div className="resrow">
        <div className="rescol">
          <h2>Inventory</h2>
          <table>
            <thead>
              <tr>
                <th>Quantity</th>
                <th>Original Price</th>
                <th>Purchase Date</th>
              </tr>
            </thead>
            <tbody>
              {inventory.map((item, index) => (
                <tr key={index}>
                  <td>{item.quantity}</td>
                  <td>{item.orig_price}</td>
                  <td>{item.purchase_date}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        
        <div className="rescol">
          <h2>Used Inventory</h2>
          <table>
            <thead>
              <tr>
                <th>Quantity</th>
                <th>Original Price</th>
                <th>Selling Price</th>
                <th>Liquidation Date</th>
              </tr>
            </thead>
            <tbody>
              {usedInventory.map((item, index) => (
                <tr key={index}>
                  <td>{item.quantity}</td>
                  <td>{item.orig_price}</td>
                  <td>{item.selling_price}</td>
                  <td>{item.liquidation_date}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
      <div className="resrow">
        <div className="rescol">
          <table>
              <thead>
                <tr>
                  <th>Inventory Value</th>
                  <th>Used Aquistion Value</th>
                  <th>Used Liquidation Value</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>{stats?.acquisition_value}</td>
                  <td>{stats?.orig_value}</td>
                  <td>{stats?.sell_value}</td>
                </tr>  
              </tbody>
          </table>
        </div>
      </div>
      <div className="excel">
        <button onClick={handleWriteToExcel}>Generate Excel Report</button>
      </div>
    </div>
  );
}

export default App;
