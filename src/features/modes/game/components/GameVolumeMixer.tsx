import { Volume2, VolumeX } from "lucide-react";
import { cn } from "../../../../shared/lib/utils";

interface GameVolumeMixerProps {
  audioMuted: boolean;
  masterVolume: number;
  musicVolume: number;
  sfxVolume: number;
  ttsVolume: number;
  ambientVolume: number;
  onMasterVolumeChange: (value: number) => void;
  onMusicVolumeChange: (value: number) => void;
  onSfxVolumeChange: (value: number) => void;
  onTtsVolumeChange: (value: number) => void;
  onAmbientVolumeChange: (value: number) => void;
  onToggleMute: () => void;
  onAudioInteract?: () => void;
  className?: string;
}

export function GameVolumeMixer({
  audioMuted,
  masterVolume,
  musicVolume,
  sfxVolume,
  ttsVolume,
  ambientVolume,
  onMasterVolumeChange,
  onMusicVolumeChange,
  onSfxVolumeChange,
  onTtsVolumeChange,
  onAmbientVolumeChange,
  onToggleMute,
  onAudioInteract,
  className,
}: GameVolumeMixerProps) {
  const rows = [
    { id: "master", label: "Master", value: masterVolume, onChange: onMasterVolumeChange },
    { id: "music", label: "Music", value: musicVolume, onChange: onMusicVolumeChange },
    { id: "sfx", label: "Sound Effects", value: sfxVolume, onChange: onSfxVolumeChange },
    { id: "tts", label: "TTS", value: ttsVolume, onChange: onTtsVolumeChange },
    { id: "ambient", label: "Ambient", value: ambientVolume, onChange: onAmbientVolumeChange },
  ];

  return (
    <div
      className={cn(
        "w-64 max-w-[calc(100vw-1.5rem)] rounded-xl border border-white/15 bg-black/85 p-3 shadow-xl backdrop-blur-md",
        className,
      )}
    >
      <div className="mb-2 flex items-center justify-between gap-3 border-b border-white/10 pb-2">
        <span className="text-[0.6875rem] font-semibold uppercase text-white/60">Volume</span>
        <button
          onClick={onToggleMute}
          className={cn(
            "flex h-7 w-7 items-center justify-center rounded-full transition-colors",
            audioMuted
              ? "bg-red-500/30 text-red-300 hover:bg-red-500/50"
              : "bg-white/10 text-white/60 hover:bg-white/20 hover:text-white",
          )}
          title={audioMuted ? "Unmute" : "Mute"}
          aria-label={audioMuted ? "Unmute" : "Mute"}
        >
          {audioMuted ? <VolumeX size={12} /> : <Volume2 size={12} />}
        </button>
      </div>

      <div className="flex flex-col gap-2.5">
        {rows.map((row) => (
          <label key={row.id} className="grid grid-cols-[5.5rem_1fr_2rem] items-center gap-2">
            <span className="truncate text-[0.6875rem] text-white/70">{row.label}</span>
            <input
              type="range"
              min={0}
              max={100}
              value={row.value}
              onPointerDown={onAudioInteract}
              onTouchStart={onAudioInteract}
              onInput={(event) => {
                onAudioInteract?.();
                row.onChange(Number(event.currentTarget.value));
              }}
              onChange={(event) => {
                onAudioInteract?.();
                row.onChange(Number(event.target.value));
              }}
              className="h-1.5 w-full cursor-pointer accent-[var(--primary)]"
            />
            <span className="text-right text-[0.6875rem] tabular-nums text-white/55">{row.value}</span>
          </label>
        ))}
      </div>
    </div>
  );
}
