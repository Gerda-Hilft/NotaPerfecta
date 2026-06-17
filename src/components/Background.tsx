import { useEffect } from "react";

function ScrewHead() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
      <defs>
        <radialGradient id="screwDome" cx="40%" cy="35%" r="50%">
          <stop offset="0%" stopColor="#6B5E50" />
          <stop offset="60%" stopColor="#4A4035" />
          <stop offset="100%" stopColor="#2A2420" />
        </radialGradient>
      </defs>
      <circle cx="8" cy="8" r="7" fill="url(#screwDome)" stroke="#1A1612" strokeWidth="0.5" />
      <line x1="4" y1="8" x2="12" y2="8" stroke="#1A1612" strokeWidth="1" strokeLinecap="round" opacity="0.6" />
      <line x1="8" y1="4" x2="8" y2="12" stroke="#1A1612" strokeWidth="1" strokeLinecap="round" opacity="0.6" />
      <circle cx="6" cy="6" r="2" fill="rgba(237,232,223,0.06)" />
    </svg>
  );
}

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
      <div className="bg-streaks" />
      <div className="bg-noise" />
      <div className="bg-screw bg-screw-tl"><ScrewHead /></div>
      <div className="bg-screw bg-screw-tr"><ScrewHead /></div>
      <div className="bg-screw bg-screw-bl"><ScrewHead /></div>
      <div className="bg-screw bg-screw-br"><ScrewHead /></div>
    </div>
  );
}
