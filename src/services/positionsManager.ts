import type { DurableObjectState } from '@cloudflare/workers-types';
import type { Env } from '../types'; // Ensure Env type is correctly imported or defined
import { z } from 'zod';

// Interface for data needed to create a new position
const PositionDataSchema = z.object({
  symbol: z.string().min(1, { message: "Symbol is required" }),
  side: z.enum(['long', 'short'], { message: "Side must be 'long' or 'short'" }),
  entryPrice: z.number({ message: "Entry price must be a number" }).positive({ message: "Entry price must be positive" }),
  size: z.number({ message: "Size must be a number" }).nonnegative({ message: "Size must be non-negative" }),
  margin: z.number().optional(),
  exchange: z.string().min(1, { message: "Exchange is required" }),
  // Add any other relevant fields for input validation here
});

// Interface for data needed to create a new position, inferred from the schema
export type PositionData = z.infer<typeof PositionDataSchema>;

// Schema for data allowed when updating a position
const UpdatePositionDataSchema = z.object({
  size: z.number().nonnegative({ message: "Size must be non-negative" }).optional(),
  margin: z.number().optional(),
  pnl: z.number().optional(),
  status: z.enum(['open', 'closing', 'closed'], { message: "Status must be 'open', 'closing', or 'closed'" }).optional(),
}).strict(); // Disallow any fields not explicitly defined here

export type UpdatePositionData = z.infer<typeof UpdatePositionDataSchema>;

// Interface for a managed position, including a unique ID
export interface ManagedPosition extends PositionData {
  id: string; // Unique identifier for the position
  createdAt: string; // ISO 8601 timestamp
  updatedAt: string; // ISO 8601 timestamp
  pnl?: number; // Profit and Loss
  status: 'open' | 'closing' | 'closed';
}

/**
 * PositionsManager Durable Object
 * Manages the state of open trading positions.
 */
export class PositionsManager implements DurableObject {
  state: DurableObjectState;
  env: Env;
  private positions!: Record<string, ManagedPosition>; // Using definite assignment assertion

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
    this.env = env;

    // Initialize state by loading from storage
    this.state.blockConcurrencyWhile(async () => {
      const storedPositions = await this.state.storage.get<Record<string, ManagedPosition>>("positions");
      if (storedPositions) {
        this.positions = storedPositions;
      } else {
        this.positions = {}; // Initialize to empty object if nothing in storage
      }
    });
  }

  private async _savePositions(): Promise<void> {
    await this.state.storage.put("positions", this.positions);
  }

  private _generateId(): string {
    // Ensure crypto.randomUUID is available in the CF Worker environment
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
      return crypto.randomUUID();
    }
    // Fallback for environments where crypto.randomUUID might not be available (e.g. older Node in local tests without shims)
    // This is a simplified UUID v4, not cryptographically strong for all uses but okay for internal IDs here.
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c: string) => {
      const r = Math.random() * 16 | 0;
      const v = c === 'x' ? r : (r & 0x3 | 0x8);
      return v.toString(16);
    });
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    const pathSegments = url.pathname.split('/').filter(Boolean); // e.g., ["positions", "some-id"]

    console.log(`[PositionsManager DO] Received request: ${request.method} ${url.pathname}`);

    try {
      if (pathSegments[0] !== 'positions') {
        return new Response("Not Found: Endpoint must start with /positions", { status: 404 });
      }

      const positionId = pathSegments[1]; // Might be undefined if not accessing a specific position

      switch (request.method) {
        case 'POST': { // Create a new position
          if (positionId) return new Response("Method Not Allowed: Cannot POST to a specific position ID", { status: 405 });
          const rawData = await request.json();
          const parseResult = PositionDataSchema.safeParse(rawData);

          if (!parseResult.success) {
            console.error("[PositionsManager DO] Validation Error:", parseResult.error.flatten());
            return new Response(JSON.stringify({ message: "Validation failed", errors: parseResult.error.flatten().fieldErrors }), { status: 400, headers: { 'Content-Type': 'application/json' } });
          }
          const newData = parseResult.data;
          const newPosition = await this.openPosition(newData);
          return new Response(JSON.stringify(newPosition), { status: 201, headers: { 'Content-Type': 'application/json' } });
        }

        case 'GET': {
          if (positionId) { // Get a specific position
            const position = await this.getPosition(positionId);
            if (position) {
              return new Response(JSON.stringify(position), { headers: { 'Content-Type': 'application/json' } });
            }
            return new Response("Position Not Found", { status: 404 });
          }
          // If no positionId, get all positions
          const allPositions = await this.getAllPositions();
          return new Response(JSON.stringify(allPositions), { headers: { 'Content-Type': 'application/json' } });
        }

        case 'PUT': { // Update an existing position
          if (!positionId) return new Response("Bad Request: Position ID required for PUT", { status: 400 });
          
          const rawUpdateData = await request.json();
          const parseResult = UpdatePositionDataSchema.safeParse(rawUpdateData);

          if (!parseResult.success) {
            console.error("[PositionsManager DO] Update Validation Error:", parseResult.error.flatten());
            return new Response(JSON.stringify({ message: "Validation failed for update", errors: parseResult.error.flatten().fieldErrors }), { status: 400, headers: { 'Content-Type': 'application/json' } });
          }
          const updateData = parseResult.data;

          // Ensure there's actually something to update if the schema allows empty objects
          if (Object.keys(updateData).length === 0) {
            return new Response(JSON.stringify({ message: "No valid fields provided for update"}), { status: 400, headers: { 'Content-Type': 'application/json' } });
          }

          const updatedPosition = await this.updatePosition(positionId, updateData);
          if (updatedPosition) {
            return new Response(JSON.stringify(updatedPosition), { headers: { 'Content-Type': 'application/json' } });
          }
          return new Response("Position Not Found or Update Failed", { status: 404 });
        }

        case 'DELETE': { // Close/delete a position
          if (!positionId) return new Response("Bad Request: Position ID required for DELETE", { status: 400 });
          const success = await this.closePosition(positionId);
          if (success) {
            return new Response("Position Closed", { status: 200 });
          }
          return new Response("Position Not Found or Close Failed", { status: 404 });
        }

        default:
          return new Response("Method Not Allowed", { status: 405 });
      }
    } catch (error: unknown) {
      let errorMessage = 'Internal Server Error';
      if (error instanceof Error) {
        errorMessage = error.message;
      }
      console.error(`[PositionsManager DO] Error: ${errorMessage}`, error instanceof Error ? error.stack : undefined);
      return new Response(`Internal Server Error: ${errorMessage}`, { status: 500 });
    }
  }

  // --- Methods for managing positions ---

  async openPosition(data: PositionData): Promise<ManagedPosition> {
    const id = this._generateId();
    const now = new Date().toISOString();
    const newPosition: ManagedPosition = {
      ...data,
      id,
      createdAt: now,
      updatedAt: now,
      status: 'open',
    };
 await this.state.blockConcurrencyWhile(async () => {
   this.positions[id] = newPosition;
   await this._savePositions();
 });
    console.log(`[PositionsManager DO] Opened position: ${id} for ${data.symbol}`);
    return newPosition;
  }

  async getAllPositions(): Promise<Record<string, ManagedPosition>> {
    return this.positions;
  }

  async getPosition(id: string): Promise<ManagedPosition | null> {
    return this.positions[id] || null;
  }

  async updatePosition(id: string, data: UpdatePositionData): Promise<ManagedPosition | null> {
    const position = this.positions[id];
    if (!position) {
      return null;
    }
    // Construct updatedPosition explicitly to ensure immutable fields are preserved
    const updatedPosition: ManagedPosition = {
      // Immutable fields from the original position
      id: position.id,
      createdAt: position.createdAt,
      symbol: position.symbol, // from PositionData
      side: position.side,     // from PositionData
      entryPrice: position.entryPrice, // from PositionData
      exchange: position.exchange, // from PositionData

      // Mutable fields: update if present in `data`, otherwise keep original
      // `data` fields are optional, so check for undefined
      size: data.size !== undefined ? data.size : position.size,
      margin: data.margin !== undefined ? data.margin : position.margin,
      pnl: data.pnl !== undefined ? data.pnl : position.pnl,
      status: data.status !== undefined ? data.status : position.status,

      // Always update `updatedAt`
      updatedAt: new Date().toISOString(),
    };
    this.positions[id] = updatedPosition;
    await this._savePositions();
    console.log(`[PositionsManager DO] Updated position: ${id}`);
    return updatedPosition;
  }

  async closePosition(id: string): Promise<boolean> {
    const position = this.positions[id];
    if (position) {
      delete this.positions[id];
      await this._savePositions();
      console.log(`[PositionsManager DO] Closed position: ${id}`);
      return true;
    }
    return false;
  }
}
