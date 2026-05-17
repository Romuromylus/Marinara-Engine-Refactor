import { AnimatePresence, motion } from "framer-motion";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type MouseEvent as ReactMouseEvent,
} from "react";
import {
  RIGHT_PANEL_WIDTH_MAX,
  RIGHT_PANEL_WIDTH_MIN,
  SIDEBAR_WIDTH_MAX,
  SIDEBAR_WIDTH_MIN,
  useUIStore,
} from "../../shared/stores/ui.store";
import { cn } from "../../shared/lib/utils";
import { CharacterLibraryView } from "../../features/characters/components/CharacterLibraryView";
import { ChatSidebar } from "./ChatSidebar";
import { RightPanel } from "./RightPanel";
import { TopBar } from "./TopBar";

function clampWidth(width: number, min: number, max: number) {
  return Math.max(min, Math.min(max, width));
}

const PANEL_RESIZE_STEP = 16;
const PANEL_RESIZE_LARGE_STEP = 48;

export function AppShell() {
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const setSidebarOpen = useUIStore((s) => s.setSidebarOpen);
  const sidebarWidth = useUIStore((s) => s.sidebarWidth);
  const setSidebarWidth = useUIStore((s) => s.setSidebarWidth);
  const rightPanelOpen = useUIStore((s) => s.rightPanelOpen);
  const rightPanelWidth = useUIStore((s) => s.rightPanelWidth);
  const characterLibraryOpen = useUIStore((s) => s.characterLibraryOpen);
  const characterDetailId = useUIStore((s) => s.characterDetailId);
  const personaDetailId = useUIStore((s) => s.personaDetailId);
  const setRightPanelWidth = useUIStore((s) => s.setRightPanelWidth);
  const closeRightPanel = useUIStore((s) => s.closeRightPanel);
  const [sidebarDragWidth, setSidebarDragWidth] = useState<number | null>(null);
  const [rightPanelDragWidth, setRightPanelDragWidth] = useState<number | null>(null);
  const sidebarDragWidthRef = useRef<number | null>(null);
  const rightPanelDragWidthRef = useRef<number | null>(null);
  const liveSidebarWidth = sidebarDragWidth ?? sidebarWidth;
  const liveRightPanelWidth = rightPanelDragWidth ?? rightPanelWidth;

  const [isMobile, setIsMobile] = useState(() => typeof window !== "undefined" && window.innerWidth < 768);
  useEffect(() => {
    const mq = window.matchMedia("(max-width: 767px)");
    const handler = (event: MediaQueryListEvent) => setIsMobile(event.matches);
    mq.addEventListener("change", handler);
    setIsMobile(mq.matches);
    return () => mq.removeEventListener("change", handler);
  }, []);

  useEffect(() => {
    if (isMobile) return;
    let rafId = 0;
    const handleResize = () => {
      cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        const { rightPanelOpen: rp, sidebarOpen: sb, sidebarWidth: sw, closeRightPanel: close } = useUIStore.getState();
        if (!rp) return;
        const panelWidth = useUIStore.getState().rightPanelWidth;
        const reserved = (sb ? sw : 0) + panelWidth;
        if (window.innerWidth - reserved < 400) close();
      });
    };
    window.addEventListener("resize", handleResize);
    return () => {
      window.removeEventListener("resize", handleResize);
      cancelAnimationFrame(rafId);
    };
  }, [isMobile]);

  const startSidebarResize = useCallback(
    (event: ReactMouseEvent<HTMLDivElement>) => {
      if (isMobile) return;
      event.preventDefault();
      const originalCursor = document.body.style.cursor;
      const originalUserSelect = document.body.style.userSelect;
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
      sidebarDragWidthRef.current = sidebarWidth;
      setSidebarDragWidth(sidebarWidth);

      const onMove = (moveEvent: MouseEvent) => {
        const nextWidth = clampWidth(moveEvent.clientX, SIDEBAR_WIDTH_MIN, SIDEBAR_WIDTH_MAX);
        sidebarDragWidthRef.current = nextWidth;
        setSidebarDragWidth(nextWidth);
      };
      let finished = false;
      const finishResize = () => {
        if (finished) return;
        finished = true;
        setSidebarWidth(sidebarDragWidthRef.current ?? useUIStore.getState().sidebarWidth);
        sidebarDragWidthRef.current = null;
        setSidebarDragWidth(null);
        document.body.style.cursor = originalCursor;
        document.body.style.userSelect = originalUserSelect;
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", finishResize);
        window.removeEventListener("blur", finishResize);
      };

      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", finishResize);
      window.addEventListener("blur", finishResize);
    },
    [isMobile, setSidebarWidth, sidebarWidth],
  );

  const startRightPanelResize = useCallback(
    (event: ReactMouseEvent<HTMLDivElement>) => {
      if (isMobile) return;
      event.preventDefault();
      const originalCursor = document.body.style.cursor;
      const originalUserSelect = document.body.style.userSelect;
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
      rightPanelDragWidthRef.current = rightPanelWidth;
      setRightPanelDragWidth(rightPanelWidth);

      const onMove = (moveEvent: MouseEvent) => {
        const nextWidth = clampWidth(
          window.innerWidth - moveEvent.clientX,
          RIGHT_PANEL_WIDTH_MIN,
          RIGHT_PANEL_WIDTH_MAX,
        );
        rightPanelDragWidthRef.current = nextWidth;
        setRightPanelDragWidth(nextWidth);
      };
      let finished = false;
      const finishResize = () => {
        if (finished) return;
        finished = true;
        setRightPanelWidth(rightPanelDragWidthRef.current ?? useUIStore.getState().rightPanelWidth);
        rightPanelDragWidthRef.current = null;
        setRightPanelDragWidth(null);
        document.body.style.cursor = originalCursor;
        document.body.style.userSelect = originalUserSelect;
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", finishResize);
        window.removeEventListener("blur", finishResize);
      };

      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", finishResize);
      window.addEventListener("blur", finishResize);
    },
    [isMobile, rightPanelWidth, setRightPanelWidth],
  );

  const adjustSidebarWidth = useCallback(
    (event: ReactKeyboardEvent<HTMLDivElement>) => {
      const step = event.shiftKey ? PANEL_RESIZE_LARGE_STEP : PANEL_RESIZE_STEP;
      let nextWidth = sidebarWidth;

      if (event.key === "ArrowLeft") nextWidth = sidebarWidth - step;
      else if (event.key === "ArrowRight") nextWidth = sidebarWidth + step;
      else if (event.key === "Home") nextWidth = SIDEBAR_WIDTH_MIN;
      else if (event.key === "End") nextWidth = SIDEBAR_WIDTH_MAX;
      else return;

      event.preventDefault();
      setSidebarWidth(clampWidth(nextWidth, SIDEBAR_WIDTH_MIN, SIDEBAR_WIDTH_MAX));
    },
    [setSidebarWidth, sidebarWidth],
  );

  const adjustRightPanelWidth = useCallback(
    (event: ReactKeyboardEvent<HTMLDivElement>) => {
      const step = event.shiftKey ? PANEL_RESIZE_LARGE_STEP : PANEL_RESIZE_STEP;
      let nextWidth = rightPanelWidth;

      if (event.key === "ArrowLeft") nextWidth = rightPanelWidth + step;
      else if (event.key === "ArrowRight") nextWidth = rightPanelWidth - step;
      else if (event.key === "Home") nextWidth = RIGHT_PANEL_WIDTH_MIN;
      else if (event.key === "End") nextWidth = RIGHT_PANEL_WIDTH_MAX;
      else return;

      event.preventDefault();
      setRightPanelWidth(clampWidth(nextWidth, RIGHT_PANEL_WIDTH_MIN, RIGHT_PANEL_WIDTH_MAX));
    },
    [rightPanelWidth, setRightPanelWidth],
  );

  return (
    <div
      data-component="AppShell"
      className={cn(
        "mari-app fixed inset-0 flex overflow-hidden bg-[var(--background)] max-md:pt-[env(safe-area-inset-top)]",
        "retro-scanlines noise-bg geometric-grid",
      )}
    >
      <div className="y2k-star hidden md:block" style={{ top: "10%", left: "5%", animationDelay: "0s" }} />
      <div className="y2k-star-md hidden md:block" style={{ top: "25%", right: "8%", animationDelay: "1.5s" }} />
      <div className="y2k-star-lg hidden md:block" style={{ top: "60%", left: "3%", animationDelay: "3s" }} />
      <div className="y2k-star hidden md:block" style={{ top: "80%", right: "12%", animationDelay: "0.8s" }} />
      <div className="y2k-star-md hidden md:block" style={{ top: "45%", left: "50%", animationDelay: "2.2s" }} />

      {sidebarOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm md:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      <aside
        data-tour="sidebar"
        data-component="ChatSidebarPanel"
        aria-label="Chat list"
        className={cn(
          "mari-sidebar flex-shrink-0 overflow-hidden bg-[var(--background)]/80 backdrop-blur-xl",
          sidebarDragWidth == null && "transition-[width] duration-200 ease-[cubic-bezier(0.16,1,0.3,1)]",
          sidebarOpen && "border-r border-[var(--sidebar-border)]/30",
          "max-md:fixed max-md:inset-y-0 max-md:left-0 max-md:z-50 max-md:shadow-2xl max-md:pt-[env(safe-area-inset-top)]",
          !sidebarOpen && "max-md:!w-0",
        )}
        style={{ width: sidebarOpen ? (isMobile ? "100vw" : liveSidebarWidth) : 0 }}
      >
        <div className="h-full" style={{ width: isMobile ? "100vw" : liveSidebarWidth }}>
          <ChatSidebar />
        </div>
      </aside>
      {!isMobile && sidebarOpen && (
        <div
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize left sidebar"
          aria-valuemin={SIDEBAR_WIDTH_MIN}
          aria-valuemax={SIDEBAR_WIDTH_MAX}
          aria-valuenow={Math.round(liveSidebarWidth)}
          tabIndex={0}
          onMouseDown={startSidebarResize}
          onKeyDown={adjustSidebarWidth}
          className="absolute inset-y-0 z-20 hidden w-1 cursor-col-resize bg-transparent transition-colors hover:bg-[var(--primary)]/30 focus-visible:bg-[var(--primary)]/40 focus-visible:outline-none md:block"
          style={{ left: sidebarOpen ? liveSidebarWidth : 0 }}
        />
      )}

      <main
        data-tour="chat-area"
        data-component="CenterContent"
        aria-label="Main content"
        className="@container mari-main relative flex min-w-0 flex-1 flex-col overflow-hidden"
      >
        {characterLibraryOpen ? (
          <CharacterLibraryView />
        ) : (
          <>
            <TopBar />
            <div className="flex flex-1 flex-col items-center justify-center overflow-hidden px-6 text-center">
              <div className="glass max-w-xl rounded-2xl p-6">
                <p className="text-sm font-medium text-[var(--foreground)]">
                  {characterDetailId
                    ? "Character editor deferred"
                    : personaDetailId
                      ? "Persona editor deferred"
                      : "Frontend shell migrated"}
                </p>
                <p className="mt-2 text-xs leading-relaxed text-[var(--muted-foreground)]">
                  {characterDetailId || personaDetailId
                    ? "The library click path is wired. The editor UI moves in a later reviewed slice."
                    : "Feature screens are intentionally deferred until their reviewed Phase 2 slices."}
                </p>
              </div>
            </div>
          </>
        )}
      </main>

      {rightPanelOpen && (
        <div className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm md:hidden" onClick={() => closeRightPanel()} />
      )}

      {isMobile ? (
        <AnimatePresence mode="wait">
          {rightPanelOpen && (
            <motion.aside
              key="mobile"
              initial={{ x: "100%" }}
              animate={{ x: 0 }}
              exit={{ x: "100%" }}
              transition={{ type: "spring", damping: 28, stiffness: 350 }}
              data-component="RightPanelMobile"
              aria-label="Settings and tools panel"
              className="mari-right-panel !fixed inset-y-0 right-0 z-50 !w-full overflow-hidden bg-[var(--background)]/80 pt-[env(safe-area-inset-top)] shadow-2xl backdrop-blur-xl"
            >
              <RightPanel />
            </motion.aside>
          )}
        </AnimatePresence>
      ) : (
        <aside
          data-component="RightPanelDesktop"
          aria-label="Settings and tools panel"
          className={cn(
            "mari-right-panel flex-shrink-0 overflow-hidden bg-[var(--background)]/80 backdrop-blur-xl",
            rightPanelDragWidth == null && "transition-[width] duration-200 ease-[cubic-bezier(0.16,1,0.3,1)]",
            rightPanelOpen && "border-l border-[var(--sidebar-border)]/30",
          )}
          style={{ width: rightPanelOpen ? liveRightPanelWidth : 0 }}
        >
          {rightPanelOpen && (
            <div className="h-full" style={{ width: liveRightPanelWidth }}>
              <RightPanel />
            </div>
          )}
        </aside>
      )}
      {!isMobile && rightPanelOpen && (
        <div
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize right sidebar"
          aria-valuemin={RIGHT_PANEL_WIDTH_MIN}
          aria-valuemax={RIGHT_PANEL_WIDTH_MAX}
          aria-valuenow={Math.round(liveRightPanelWidth)}
          tabIndex={0}
          onMouseDown={startRightPanelResize}
          onKeyDown={adjustRightPanelWidth}
          className="absolute inset-y-0 z-20 hidden w-1 cursor-col-resize bg-transparent transition-colors hover:bg-[var(--primary)]/30 focus-visible:bg-[var(--primary)]/40 focus-visible:outline-none md:block"
          style={{ right: rightPanelOpen ? liveRightPanelWidth : 0 }}
        />
      )}
    </div>
  );
}
