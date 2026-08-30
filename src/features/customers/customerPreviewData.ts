export type CustomerPreview = {
  id: number;
  name: string;
  phone: string;
  notes?: string;
  motorcycles: MotorcyclePreview[];
  serviceHistory: ServiceHistoryPreview[];
};

export type MotorcyclePreview = {
  id: number;
  make: string;
  model: string;
  year?: number;
  color: string;
  plate?: string;
  vin?: string;
  chassis?: string;
};

export type ServicePartPreview = {
  id: number;
  name: string;
  quantityLabel: string;
  unitPriceFils: number;
  lineTotalFils: number;
};

export type ServiceHistoryPreview = {
  id: number;
  date: string;
  motorcycleId: number;
  odometerKm: number | null;

  complaint: string;
  diagnosis?: string;
  workPerformed?: string;

  laborChargeFils: number;
  parts: ServicePartPreview[];

  status: "OPEN" | "READY_FOR_PICKUP" | "CLOSED" | "CANCELLED";
  totalFils: number;
};



export const previewCustomers: CustomerPreview[] = [
  {
    id: 1,
    name: "Ahmad Ali",
    phone: "+962791234567",
    motorcycles: [
      {
        id: 1,
        make: "Honda",
        model: "CB150R",
        year: 2022,
        color: "Black",
        plate: "29-12345",
        vin: "MLHPC...",
      },
      {
        id: 2,
        make: "Yamaha",
        model: "YBR125",
        year: 2020,
        color: "Red",
        vin: "ME1RG...",
      },
    ],
    serviceHistory: [
        {
        id: 125,
        date: "Aug 30, 2026",
        motorcycleId: 1,
        odometerKm: 15_870,

        complaint: "Engine makes a ticking noise when hot.",
        diagnosis: undefined,
        workPerformed: undefined,

        laborChargeFils: 0,
        parts: [],

        status: "OPEN",
        totalFils: 0,
        },
      {
  id: 104,
  date: "Aug 28, 2026",
  motorcycleId: 1,
  odometerKm: 15_200,

  complaint: "Oil leak",
  diagnosis: "Oil filter seal was leaking under pressure.",
  workPerformed: "Replaced oil filter and changed engine oil.",

  laborChargeFils: 5_000,

  parts: [
    {
      id: 1,
      name: "Oil Filter",
      quantityLabel: "1 Piece",
      unitPriceFils: 4_500,
      lineTotalFils: 4_500,
    },
    {
      id: 2,
      name: "10W40 Engine Oil",
      quantityLabel: "2.000 L",
      unitPriceFils: 6_000,
      lineTotalFils: 12_000,
    },
  ],

  status: "CLOSED",
  totalFils: 21_500,
},
      {
  id: 87,
  date: "Jun 14, 2026",
  motorcycleId: 1,
  odometerKm: null,

  complaint: "Front brake noise",
  diagnosis: "Front brake pads were worn.",
  workPerformed: "Replaced front brake pads and inspected the brake system.",

  laborChargeFils: 5_000,

  parts: [
    {
      id: 3,
      name: "Front Brake Pads",
      quantityLabel: "1 Piece",
      unitPriceFils: 9_000,
      lineTotalFils: 9_000,
    },
  ],

  status: "CLOSED",
  totalFils: 14_000,
},
    ],
  },
  {
    id: 2,
    name: "Omar Khaled",
    phone: "+962785555555",
    motorcycles: [
      {
        id: 3,
        make: "Suzuki",
        model: "GSX150",
        year: 2021,
        color: "Blue",
        plate: "31-92841",
      },
    ],
    serviceHistory: [],
  },
  {
    id: 3,
    name: "Yousef Mahmoud",
    phone: "+962799876543",
    motorcycles: [],
    serviceHistory: [],
  },
  {
    id: 4,
    name: "ليث أحمد",
    phone: "+962777654321",
    motorcycles: [],
    serviceHistory: [],
  },
];

