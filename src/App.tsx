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

interface AllTransactions {
  id: number | null;
  quantity: number | null;
  orig_price: number | null;
  selling_price: number | null;
  purchase_date: string | null;
  liquidation_date: string | null;
  is_used: boolean;
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
  const [showPopup, setShowPopup] = useState(false);
  const [transactions, setTransactions] = useState<AllTransactions[]>([]);
  const [editingTransaction, setEditingTransaction] = useState<AllTransactions | null>(null);

  useEffect(() => {
    fetchInventory();
    fetchUsedInventory();
    fetchStats();
    fetchAllTransactions();

  }, []);

  async function fetchInventory() {
    const inventory = await invoke("print_inventory") as TaoPurchase[];
    setInventory(inventory);
  }
  const fetchAllTransactions = async () => {
    const allTransactions = await invoke("show_all_transactions") as AllTransactions[];
    console.log("These are all my transactions ",allTransactions);
    setTransactions(allTransactions);
  };

  const handleDeleteTransaction = async (id: number) => {
    await invoke("remove_transaction_via_id", { id });
    fetchAllTransactions();
  };

  const handleEditTransactionStart = (transaction: AllTransactions) => {
    setEditingTransaction(transaction);
    setShowPopup(true);
  };

  const handleApplyChanges = async () => {
    // Logic to add or edit a transaction based on `editingTransaction`
    // For example, if editingTransaction is null, add a new transaction
    // Otherwise, edit the existing transaction in the database

    // After applying changes, refresh and close the popup
    await invoke("redo_transactions");
    fetchAllTransactions();
    setShowPopup(false);
    setEditingTransaction(null); // Reset editing state
  };

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
      quantity: parseFloat(quantity), 
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
      quantityNeeded: parseFloat(quantityNeeded), 
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
                  <td>{item.quantity ? parseFloat(item.quantity.toFixed(2)) : '0.00'}</td>
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
                  <td>{item.quantity ? parseFloat(item.quantity.toFixed(2)) : '0.00'}</td>
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
                <th>Used Acquisition Value</th>
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
      <button onClick={() => setShowPopup(true)}>View & Manage Transactions</button>
      {showPopup && (
        <div className="popup">
          <h2>All Transactions</h2>
          {transactions.map((transaction) => (
            <div key={transaction.id}>
              <button onClick={() => handleEditTransactionStart(transaction)}>Edit</button>
              <button onClick={() => {
                if (transaction.id !== null) {
                  handleDeleteTransaction(transaction.id);
                }
              }}>Delete</button>
            </div>
          ))}
          <button onClick={() => setShowPopup(false)}>Close</button>
        </div>
      )}
    </div>
  );
}



export default App;
