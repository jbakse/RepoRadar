import { FilterType } from "../types";

interface Props {
  filter: FilterType;
  onFilterChange: (filter: FilterType) => void;
  repoCount: number;
  filteredCount: number;
}

const FILTERS: { value: FilterType; label: string }[] = [
  { value: "all", label: "All" },
  { value: "dirty", label: "Dirty" },
  { value: "unpushed", label: "Unpushed" },
  { value: "missing_remote", label: "No Remote" },
  { value: "high_risk", label: "High Risk" },
];

export function FilterBar({
  filter,
  onFilterChange,
  repoCount,
  filteredCount,
}: Props) {
  if (repoCount === 0) return null;

  return (
    <div className="filter-bar">
      <div className="filter-buttons">
        {FILTERS.map((f) => (
          <button
            key={f.value}
            className={`filter-btn ${filter === f.value ? "active" : ""}`}
            onClick={() => onFilterChange(f.value)}
          >
            {f.label}
          </button>
        ))}
      </div>
      <span className="filter-count">
        {filteredCount === repoCount
          ? `${repoCount} repositories`
          : `${filteredCount} of ${repoCount} repositories`}
      </span>
    </div>
  );
}
