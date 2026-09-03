import { type FormEvent, useCallback, useEffect, useRef, useState } from "react";
import { Bike, Search } from "lucide-react";
import { searchMotorcycleDirectory } from "./api/motorcycleDirectoryApi";
import type { MotorcycleDirectoryEntry } from "./api/motorcycleDirectoryApi.types";
import "./MotorcyclesPage.css";

const DIRECTORY_LIMIT = 50;

export function MotorcyclesPage({ onSelectMotorcycle }: { onSelectMotorcycle: (id: number) => void }) {
  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const [motorcycles, setMotorcycles] = useState<MotorcycleDirectoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const request = useRef(0);
  const load = useCallback(async () => {
    const current = ++request.current;
    setLoading(true); setError(false);
    try {
      const result = await searchMotorcycleDirectory({ query: submittedQuery, limit: DIRECTORY_LIMIT });
      if (request.current === current) setMotorcycles(result);
    } catch {
      if (request.current === current) { setMotorcycles([]); setError(true); }
    } finally { if (request.current === current) setLoading(false); }
  }, [submittedQuery]);
  useEffect(() => { void load(); return () => { request.current += 1; }; }, [load]);
  function submit(event: FormEvent) { event.preventDefault(); setSubmittedQuery(query.trim()); }
  return <section className="motorcycles-directory">
    <div className="page-header"><div><h1>Motorcycles</h1><p>Workshop motorcycles, owners, and service activity.</p></div></div>
    <div className="content-panel">
      <form className="motorcycles-search" onSubmit={submit}><label><Search size={17}/><input type="search" aria-label="Search Motorcycles" value={query} onChange={(e)=>setQuery(e.target.value)} placeholder="Plate, VIN, chassis, make, model, customer..."/></label><button className="secondary-button" type="submit">Search</button></form>
      {error ? <div className="empty-state" role="alert"><strong>Motorcycles could not be loaded</strong><span>Please try again.</span></div> : <div className="table-wrapper"><table className="data-table motorcycles-table"><thead><tr><th>Motorcycle</th><th>Identity</th><th>Owner</th><th>Latest visit</th><th>Active work</th></tr></thead><tbody>
        {motorcycles.map((m)=><tr key={m.id} role="button" tabIndex={0} aria-label={`Open Motorcycle ${m.id}`} onClick={()=>onSelectMotorcycle(m.id)} onKeyDown={(e)=>{if(e.key==="Enter"||e.key===" "){e.preventDefault();onSelectMotorcycle(m.id);}}}>
          <td><strong>{m.makeName} {m.model}</strong><span className="motorcycle-secondary">#{m.id} · {m.year ?? "Year not recorded"} · {m.colorName}</span></td>
          <td><strong>{m.plateNumber ?? "No plate"}</strong><span className="motorcycle-secondary">VIN {m.vin ?? "—"} · Chassis {m.chassisNumber ?? "—"}</span></td>
          <td><strong>{m.ownerName}</strong><span className="motorcycle-secondary">{m.ownerPhone}</span></td>
          <td>{m.latestServiceVisitAt === null ? "No visits" : new Date(m.latestServiceVisitAt).toLocaleString()}</td>
          <td>{m.activeServiceVisitId === null ? "—" : <span className="status-badge status-open">#{m.activeServiceVisitId} {m.activeServiceVisitStatus?.replace(/_/g, " ")}</span>}</td>
        </tr>)}
      </tbody></table>{loading&&<div className="empty-state"><strong>Loading Motorcycles...</strong></div>}{!loading&&motorcycles.length===0&&<div className="empty-state"><Bike size={24}/><strong>No Motorcycles found</strong><span>Try a different persisted identifier or owner.</span></div>}</div>}
    </div>
  </section>;
}
