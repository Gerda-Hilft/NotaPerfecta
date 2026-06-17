import { useEffect } from "react";

export function Background() {
  useEffect(() => {
    function onMove(e: PointerEvent) {
      const x = (e.clientX / window.innerWidth - 0.5) * 40;
      const y = (e.clientY / window.innerHeight - 0.5) * 40;
      document.documentElement.style.setProperty("--pointer-x", x.toFixed(2));
      document.documentElement.style.setProperty("--pointer-y", y.toFixed(2));
    }
    window.addEventListener("pointermove", onMove, { passive: true });
    return () => window.removeEventListener("pointermove", onMove);
  }, []);

  return (
    <div className="bg-field" aria-hidden="true">
      <div className="bg-orb orb-1" />
      <div className="bg-orb orb-2" />
      <div className="bg-orb orb-3" />
      <div className="bg-grid" />
      <div className="bg-noise" />
    </div>
  );
}
