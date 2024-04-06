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

interface Spec {
  quantity: number;
  orig_price: number;
  sale_price: number;
  liquidation_date: string;
}

function App() {
  const [inventory, setInventory] = useState<TaoPurchase[]>([]);
  const [usedInventory, setUsedInventory] = useState<TaoPurchase[]>([]);
  const [quantityNeeded, setQuantityNeeded] = useState('');
  const [liquidationDateTime, setLiquidationDateTime] = useState('');
  const [quantity, setQuantity] = useState('');
  const [salePrice, setSalePrice] = useState('');
  const [price_per_ton, set_price_per_ton] = useState('');
  const [date, setDate] = useState('');

  useEffect(() => {
    fetchInventory();
    fetchUsedInventory();
  }, []);

  async function fetchInventory() {
    const inventory = await invoke("print_inventory") as TaoPurchase[];
    setInventory(inventory);
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
      <button onClick={handleWriteToExcel}>Generate Excel Report</button>
      <form onSubmit={handleRecordPurchase}>
        <input type="number" value={quantity} onChange={(e) => setQuantity(e.target.value)} placeholder="Quantity" />
        <input type="number" value={price_per_ton} onChange={(e) => set_price_per_ton(e.target.value)} placeholder="Price Per Tao" />
        <input type="datetime-local" value={date} onChange={(e) => setDate(e.target.value)} />
        <button type="submit">Record Purchase</button>
      </form>
      <form onSubmit={handleUseTao}>
        <input type="number" value={quantityNeeded} onChange={(e) => setQuantityNeeded(e.target.value)} placeholder="Quantity Needed" />
        <input type="number" value={salePrice} onChange={(e) => setSalePrice(e.target.value)} placeholder="Selling Price" />
        <input type="datetime-local" value={liquidationDateTime} onChange={(e) => setLiquidationDateTime(e.target.value)} placeholder="Liquidation Date" />
        <button type="submit">Use Tao</button>
      </form>
      <div>
        <h2>Inventory</h2>
        {inventory.map((item, index) => (
          <div key={index}>
            Quantity: {item.quantity}, Price Per Tao: {item.orig_price}, Purchase Date: {item.purchase_date}
          </div>
        ))}
      </div>
      <div>
        <h2>Used Inventory</h2>
        {usedInventory.map((item, index) => (
          <div key={index}>
            Quantity: {item.quantity}, Original Price: {item.orig_price} ,Selling Price: {item.selling_price}, Liquidation Date: {item.liquidation_date}
          </div>
        ))}
      </div>
    </div>
  );
}

export default App;
