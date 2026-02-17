interface ScanProgressBarProps {
  phase: "discovering" | "analyzing";
  total: number;
  completed: number;
  currentRepo: string | null;
}

export function ScanProgressBar({
  phase,
  total,
  completed,
  currentRepo,
}: ScanProgressBarProps) {
  const percentage = total > 0 ? Math.round((completed / total) * 100) : 0;

  return (
    <div className="scan-progress">
      <div className="scan-progress-bar">
        {phase === "discovering" ? (
          <div className="scan-progress-fill indeterminate" />
        ) : (
          <div
            className="scan-progress-fill"
            style={{ width: `${percentage}%` }}
          />
        )}
      </div>
      <div className="scan-progress-text">
        {phase === "discovering" ? (
          <>Discovering repositories... ({total} found)</>
        ) : (
          <>
            Analyzing {currentRepo || "..."} ({completed} of {total})
          </>
        )}
      </div>
    </div>
  );
}
