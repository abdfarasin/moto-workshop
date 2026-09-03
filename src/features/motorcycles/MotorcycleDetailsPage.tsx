import { useEffect, useRef, useState } from "react";
import { ArrowLeft, Gauge, Hash, Palette, User, Wrench } from "lucide-react";
import { loadServiceVisitWorkspace } from "../service/api/serviceVisitApi";
import type { ServiceVisitWorkspace } from "../service/api/serviceVisitApi";
import { NewServiceVisitDialog } from "../service/new-visit/NewServiceVisitDialog";
import { loadMotorcycleDetails } from "./api/motorcycleDirectoryApi";
import type { MotorcycleDetails } from "./api/motorcycleDirectoryApi.types";
import "./MotorcycleDetailsPage.css";

type Props = { motorcycleId:number; onBack:()=>void; onOpenCustomer:(id:number)=>void; onOpenServiceVisit:(workspace:ServiceVisitWorkspace)=>void };

export function MotorcycleDetailsPage({motorcycleId,onBack,onOpenCustomer,onOpenServiceVisit}:Props){
  const [details,setDetails]=useState<MotorcycleDetails|null>(null);
  const [loading,setLoading]=useState(true);
  const [error,setError]=useState(false);
  const [workspaceError,setWorkspaceError]=useState(false);
  const [newVisitOpen,setNewVisitOpen]=useState(false);
  const request=useRef(0);
  useEffect(()=>{
    const current=++request.current;
    setLoading(true); setError(false);
    void loadMotorcycleDetails(motorcycleId)
      .then((value)=>{if(current===request.current)setDetails(value);})
      .catch(()=>{if(current===request.current)setError(true);})
      .finally(()=>{if(current===request.current)setLoading(false);});
    return()=>{request.current+=1;};
  },[motorcycleId]);
  async function openVisit(id:number){setWorkspaceError(false);try{onOpenServiceVisit(await loadServiceVisitWorkspace(id));}catch{setWorkspaceError(true);}}
  if(loading)return <div className="empty-state"><strong>Loading Motorcycle...</strong></div>;
  if(error||details===null)return <div className="empty-state" role="alert"><strong>Motorcycle could not be loaded</strong><button type="button" className="secondary-button" onClick={onBack}>Back to Motorcycles</button></div>;
  return <section className="motorcycle-details-page">
    <button type="button" className="back-button" onClick={onBack}><ArrowLeft size={17}/>Motorcycles</button>
    <div className="motorcycle-profile-header"><div><h1>{details.makeName} {details.model}</h1><button type="button" className="motorcycle-owner-button" onClick={()=>onOpenCustomer(details.ownerCustomerId)}><User size={15}/>{details.ownerName} · {details.ownerPhone}</button></div>
      <div className="header-actions">{details.activeServiceVisitId===null?<button type="button" className="primary-button service-action" onClick={()=>setNewVisitOpen(true)}><Wrench size={17}/>New Service Visit</button>:<button type="button" className="primary-button service-action" onClick={()=>void openVisit(details.activeServiceVisitId!)} aria-label={`Open active Service Visit ${details.activeServiceVisitId}`}><Wrench size={17}/>Open active Visit #{details.activeServiceVisitId}</button>}</div>
    </div>
    {workspaceError&&<p role="alert">Could not open this Service Visit. Please try again.</p>}
    <div className="motorcycle-info-grid"><Info icon={<Hash size={18}/>} label="Plate" value={details.plateNumber}/><Info icon={<Gauge size={18}/>} label="Year" value={details.year?.toString()??null}/><Info icon={<Palette size={18}/>} label="Color" value={details.colorName}/></div>
    <section className="vehicle-identification-panel"><div className="section-header compact-section-header"><div><h2>Identification</h2><p>Recorded identifiers for this motorcycle.</p></div></div><div className="vehicle-identity-list"><div className="identity-row"><span>VIN</span><strong>{details.vin??"Not recorded"}</strong></div><div className="identity-row"><span>Chassis number</span><strong>{details.chassisNumber??"Not recorded"}</strong></div></div></section>
    <section className="details-section"><div className="section-header"><div><h2>Service History</h2><p>Newest workshop visits for this motorcycle.</p></div></div><div className="content-panel"><div className="table-wrapper"><table className="data-table"><thead><tr><th>Visit</th><th>Date</th><th>Mileage</th><th>Complaint</th><th>Status</th><th className="money-column">Total</th></tr></thead><tbody>{details.serviceHistory.map((visit)=><tr key={visit.id} role="button" tabIndex={0} aria-label={`Open Service Visit ${visit.id}`} onClick={()=>void openVisit(visit.id)} onKeyDown={(event)=>{if(event.key==="Enter"||event.key===" "){event.preventDefault();void openVisit(visit.id);}}}><td><strong>#{visit.id}</strong></td><td>{new Date(visit.openedAt).toLocaleString()}</td><td>{visit.odometerKm===null?"Not recorded":`${visit.odometerKm.toLocaleString()} km`}</td><td>{visit.customerComplaint}</td><td><span className={`status-badge status-${visit.status.toLowerCase()}`}>{visit.status.replace(/_/g," ")}</span></td><td className="money-column">{(visit.totalFils/1000).toFixed(3)} JD</td></tr>)}</tbody></table>{details.serviceHistory.length===0&&<div className="empty-state"><Wrench size={24}/><strong>No service history</strong></div>}</div></div></section>
    <NewServiceVisitDialog open={newVisitOpen} initialCustomer={{id:details.ownerCustomerId,name:details.ownerName,phone:details.ownerPhone}} initialMotorcycleId={details.id} onClose={()=>setNewVisitOpen(false)} onCreated={onOpenServiceVisit}/>
  </section>;
}
function Info({icon,label,value}:{icon:React.ReactNode;label:string;value:string|null}){return <div className="info-card"><div className="info-card-icon">{icon}</div><div><span className="info-label">{label}</span><strong>{value??"Not recorded"}</strong></div></div>}
