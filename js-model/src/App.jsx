import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import DB from "./pachadb/db.js";
import Query from "./pachadb/query.js";
import "./App.css";

function App() {
  const [count, setCount] = useState(0);
  useEffect(() => {
    // Example usage
    const db = DB.createDB();

    // Add some sample data
    const updatedDb = DB.insert(db, "todos", [
      { title: "Code a bunch", status: "In Progress" },
      { title: "Review PRs", status: "Todo" },
      { title: "Write documentation", status: "Done" },
    ]);

    console.log("todos db", JSON.stringify(updatedDb, null, 2));

    // Define a query
    const query = {
      todos: {
        $: {
          where: {
            or: [{ title: "Code a bunch" }, { title: "Review PRs" }],
          },
        },
      },
    };

    const results = Query.executeQuery(updatedDb, query);
    console.log("results", JSON.stringify(results, null, 2));
  }, []);
  return (
    <>
      <div>
        <a href="https://vitejs.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>count is {count}</button>
        <p>
          Edit <code>src/App.jsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">Click on the Vite and React logos to learn more</p>
    </>
  );
}

export default App;
