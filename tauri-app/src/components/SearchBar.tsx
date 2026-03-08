import { forwardRef } from "react";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  resultCount?: number;
}

export const SearchBar = forwardRef<HTMLInputElement, SearchBarProps>(
  ({ value, onChange, resultCount }, ref) => {
    return (
      <div className="flex items-center gap-3">
        <div className="relative flex-1 max-w-[340px] group">
          {/* Search icon */}
          <div className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-cyan-400 transition-colors duration-200 text-xs pointer-events-none">
            &#x1F50D;
          </div>
          <input
            ref={ref}
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder="Search files... (Ctrl+F)"
            className="w-full glass rounded-lg pl-9 pr-8 py-2 text-sm text-white placeholder-gray-600
                       border border-white/[0.06] focus:border-cyan-500/40 focus:glow-cyan-soft
                       focus:outline-none transition-all duration-200"
          />
          {value && (
            <button
              onClick={() => onChange("")}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white
                         w-5 h-5 flex items-center justify-center rounded-full hover:bg-white/10
                         transition-all duration-150 text-xs"
            >
              &#x2715;
            </button>
          )}
        </div>
        {resultCount !== undefined && (
          <span className="text-[11px] text-gray-500 font-mono bg-white/[0.03] px-2 py-1 rounded-md">
            {resultCount} result{resultCount !== 1 ? "s" : ""}
          </span>
        )}
      </div>
    );
  }
);
