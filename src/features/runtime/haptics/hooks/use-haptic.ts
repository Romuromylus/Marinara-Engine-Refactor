// ──────────────────────────────────────────────
// Hook: Haptic Feedback (Buttplug.io / Intiface Central)
// ──────────────────────────────────────────────
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invokeTauri } from "../../../../shared/api/tauri-client";
import type { HapticStatus, HapticDeviceCommand } from "../../../../engine/contracts/types/haptic";

const HAPTIC_KEY = ["haptic", "status"] as const;
export const HAPTIC_INTIFACE_URL_STORAGE_KEY = "marinara_haptic_intiface_url";

/** Current haptic connection status and devices. */
export function useHapticStatus() {
  return useQuery<HapticStatus>({
    queryKey: HAPTIC_KEY,
    queryFn: () => invokeTauri<HapticStatus>("haptic_status"),
    refetchInterval: () => (document.hidden ? false : 15_000), // Pause while tab is hidden
  });
}

/** Connect to Intiface Central. */
export function useHapticConnect() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (url?: string) => invokeTauri<HapticStatus>("haptic_connect", { body: url ? { url } : null }),
    onSuccess: () => qc.invalidateQueries({ queryKey: HAPTIC_KEY }),
  });
}

/** Disconnect from Intiface Central. */
export function useHapticDisconnect() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => invokeTauri<HapticStatus>("haptic_disconnect"),
    onSuccess: () => qc.invalidateQueries({ queryKey: HAPTIC_KEY }),
  });
}

/** Start scanning for devices. */
export function useHapticStartScan() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => invokeTauri("haptic_start_scan"),
    onSuccess: () => qc.invalidateQueries({ queryKey: HAPTIC_KEY }),
  });
}

/** Stop scanning for devices. */
export function useHapticStopScan() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => invokeTauri("haptic_stop_scan"),
    onSuccess: () => qc.invalidateQueries({ queryKey: HAPTIC_KEY }),
  });
}

/** Send a manual command to a device. */
export function useHapticCommand() {
  return useMutation({
    mutationFn: (command: HapticDeviceCommand) => invokeTauri("haptic_command", { command }),
  });
}

/** Stop all devices. */
export function useHapticStopAll() {
  return useMutation({
    mutationFn: () => invokeTauri("haptic_stop_all"),
  });
}
