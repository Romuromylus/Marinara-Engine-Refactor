import {
  trackerSnapshotApi,
  type TrackerSnapshot,
  type TrackerSnapshotInput,
  type TrackerSnapshotTarget,
} from "../../../../shared/api/tracker-snapshot-api";

export type RoleplayTrackerSnapshot = TrackerSnapshot;
export type RoleplayTrackerSnapshotInput = TrackerSnapshotInput;

export const roleplayTrackerApi = {
  latest(chatId: string): Promise<RoleplayTrackerSnapshot | null> {
    return trackerSnapshotApi.latest(chatId);
  },

  getTurn(chatId: string, target: TrackerSnapshotTarget): Promise<RoleplayTrackerSnapshot | null> {
    return trackerSnapshotApi.get(chatId, target);
  },

  saveTurn(chatId: string, snapshot: RoleplayTrackerSnapshotInput): Promise<RoleplayTrackerSnapshot> {
    return trackerSnapshotApi.save(chatId, snapshot);
  },
};
