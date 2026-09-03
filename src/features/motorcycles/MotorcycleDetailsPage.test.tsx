// @vitest-environment jsdom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { loadMotorcycleDetails } from "./api/motorcycleDirectoryApi";
import { loadServiceVisitWorkspace } from "../service/api/serviceVisitApi";
import { MotorcycleDetailsPage } from "./MotorcycleDetailsPage";

vi.mock("./api/motorcycleDirectoryApi", () => ({ loadMotorcycleDetails: vi.fn() }));
vi.mock("../service/api/serviceVisitApi", async () => ({ ...(await vi.importActual("../service/api/serviceVisitApi")), loadServiceVisitWorkspace: vi.fn() }));
vi.mock("../service/new-visit/NewServiceVisitDialog", () => ({ NewServiceVisitDialog: () => null }));

const detailsMock = vi.mocked(loadMotorcycleDetails);
const workspaceMock = vi.mocked(loadServiceVisitWorkspace);

describe("MotorcycleDetailsPage", () => {
  beforeEach(() => {
    detailsMock.mockReset(); workspaceMock.mockReset();
    detailsMock.mockResolvedValue({ id:11, makeName:"Honda", model:"CB150R", year:2022, colorName:"Black", plateNumber:"29-12345", vin:"JH2RC4468MK123456", chassisNumber:null, ownerCustomerId:7, ownerName:"Ahmad Ali", ownerPhone:"+962791234567", activeServiceVisitId:31, activeServiceVisitStatus:"OPEN", serviceHistory:[{ id:31, openedAt:2000, odometerKm:42000, customerComplaint:"Oil leak", status:"OPEN", totalFils:9500 }, { id:30, openedAt:1000, odometerKm:null, customerComplaint:"Older repair", status:"CLOSED", totalFils:5000 }] });
    workspaceMock.mockResolvedValue({ visit:{id:31} } as never);
  });
  afterEach(() => cleanup());

  it("loads by persisted ID and opens owner and real workspace routes", async () => {
    // Arrange
    const user=userEvent.setup(); const onOwner=vi.fn(); const onVisit=vi.fn();
    render(<MotorcycleDetailsPage motorcycleId={11} onBack={vi.fn()} onOpenCustomer={onOwner} onOpenServiceVisit={onVisit} />);
    expect(await screen.findByText("Honda CB150R")).toBeTruthy();
    expect(detailsMock).toHaveBeenCalledWith(11);

    // Act
    await user.click(screen.getByRole("button", {name:/Ahmad Ali/}));
    await user.click(screen.getByRole("button", {name:"Open Service Visit 30"}));
    await user.click(screen.getByRole("button", {name:"Open active Service Visit 31"}));

    // Assert
    expect(onOwner).toHaveBeenCalledWith(7);
    expect(workspaceMock).toHaveBeenCalledWith(30);
    expect(workspaceMock).toHaveBeenCalledWith(31);
    await waitFor(()=>expect(onVisit).toHaveBeenCalled());
  });
});
