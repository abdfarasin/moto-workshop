import type { ServiceVisitStatus } from "../../service/api/serviceVisitApi";

export type MotorcycleDirectoryCommandErrorCategory =
  | "motorcycleNotFound"
  | "databaseError";

export interface MotorcycleDirectoryCommandErrorPayload {
  category: MotorcycleDirectoryCommandErrorCategory;
  message: string;
}

export interface SearchMotorcycleDirectoryInput {
  query: string;
  limit?: number;
}

export interface MotorcycleDirectoryEntry {
  id: number;
  makeName: string;
  model: string;
  year: number | null;
  colorName: string;
  plateNumber: string | null;
  vin: string | null;
  chassisNumber: string | null;
  ownerCustomerId: number;
  ownerName: string;
  ownerPhone: string;
  latestServiceVisitAt: number | null;
  activeServiceVisitId: number | null;
  activeServiceVisitStatus: ServiceVisitStatus | null;
}

export interface MotorcycleServiceHistoryEntry {
  id: number;
  openedAt: number;
  odometerKm: number | null;
  customerComplaint: string;
  status: ServiceVisitStatus;
  totalFils: number;
}

export interface MotorcycleDetails {
  id: number;
  makeName: string;
  model: string;
  year: number | null;
  colorName: string;
  plateNumber: string | null;
  vin: string | null;
  chassisNumber: string | null;
  ownerCustomerId: number;
  ownerName: string;
  ownerPhone: string;
  activeServiceVisitId: number | null;
  activeServiceVisitStatus: ServiceVisitStatus | null;
  serviceHistory: MotorcycleServiceHistoryEntry[];
}
