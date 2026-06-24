import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";

export type NotificationType = "error" | "success" | "info" | "warning";

export interface Notification {
  id: string;
  title: string;
  message?: string;
  type: NotificationType;
  duration: number;
}

interface NotificationContext {
  notify: (opts: {
    title: string;
    message?: string;
    type: NotificationType;
    duration?: number;
  }) => void;
}

const Ctx = createContext<NotificationContext | null>(null);

const MAX_VISIBLE = 5;
const DEFAULT_DURATION = 5000;

let nextId = 0;

export function NotificationProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Notification[]>([]);
  const [dismissing, setDismissing] = useState<Set<string>>(new Set());

  const dismiss = useCallback((id: string) => {
    setDismissing((prev) => new Set(prev).add(id));
    setTimeout(() => {
      setDismissing((prev) => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 200);
  }, []);

  const notify = useCallback(
    (opts: {
      title: string;
      message?: string;
      type: NotificationType;
      duration?: number;
    }) => {
      const id = `toast-${++nextId}`;
      const toast: Notification = {
        id,
        title: opts.title,
        message: opts.message,
        type: opts.type,
        duration: opts.duration ?? DEFAULT_DURATION,
      };
      setToasts((prev) => {
        const next = [...prev, toast];
        if (next.length > MAX_VISIBLE) {
          const overflow = next.slice(0, next.length - MAX_VISIBLE);
          for (const t of overflow) dismiss(t.id);
        }
        return next;
      });
    },
    [dismiss],
  );

  return (
    <Ctx.Provider value={{ notify }}>
      {children}
      {createPortal(
        <div className="notification-container" role="region" aria-label="Notifications">
          {toasts.map((t) => (
            <Toast
              key={t.id}
              toast={t}
              leaving={dismissing.has(t.id)}
              onDismiss={dismiss}
            />
          ))}
        </div>,
        document.body,
      )}
    </Ctx.Provider>
  );
}

const ICONS: Record<NotificationType, string> = {
  error: "⚠️",
  success: "✅",
  info: "ℹ️",
  warning: "⚠️",
};

function Toast({
  toast,
  leaving,
  onDismiss,
}: {
  toast: Notification;
  leaving: boolean;
  onDismiss: (id: string) => void;
}) {
  const remainingRef = useRef(toast.duration);
  const startRef = useRef<number | null>(null);

  useEffect(() => {
    let raf: number;

    function tick() {
      if (
        document.visibilityState !== "visible" ||
        !document.hasFocus()
      ) {
        startRef.current = null;
        raf = requestAnimationFrame(tick);
        return;
      }

      const now = performance.now();
      if (startRef.current === null) {
        startRef.current = now;
      }

      const elapsed = now - startRef.current;
      if (elapsed >= remainingRef.current) {
        onDismiss(toast.id);
        return;
      }

      raf = requestAnimationFrame(tick);
    }

    raf = requestAnimationFrame(tick);

    function onVisibility() {
      if (document.visibilityState === "hidden" && startRef.current !== null) {
        remainingRef.current -= performance.now() - startRef.current;
        startRef.current = null;
      }
    }

    function onBlur() {
      if (startRef.current !== null) {
        remainingRef.current -= performance.now() - startRef.current;
        startRef.current = null;
      }
    }

    document.addEventListener("visibilitychange", onVisibility);
    window.addEventListener("blur", onBlur);

    return () => {
      cancelAnimationFrame(raf);
      document.removeEventListener("visibilitychange", onVisibility);
      window.removeEventListener("blur", onBlur);
    };
  }, [toast.id, onDismiss]);

  return (
    <div
      className={`notification-toast notification-toast--${toast.type}${leaving ? " notification-toast--leaving" : ""}`}
      role="alert"
    >
      <button
        className="notification-close"
        onClick={() => onDismiss(toast.id)}
        aria-label="Schließen"
      >
        &times;
      </button>
      <span className="notification-icon">{ICONS[toast.type]}</span>
      <div className="notification-body">
        <strong className="notification-title">{toast.title}</strong>
        {toast.message && (
          <p className="notification-message">{toast.message}</p>
        )}
      </div>
    </div>
  );
}

export function useNotifications() {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useNotifications must be used within NotificationProvider");
  return ctx;
}
