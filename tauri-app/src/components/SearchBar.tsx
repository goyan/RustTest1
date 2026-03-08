import { forwardRef } from "react";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  resultCount?: number;
}

export const SearchBar = forwardRef<HTMLInputElement, SearchBarProps>(
  ({ value, onChange, resultCount }, ref) => {
    return (
      <div className="flex items-center gap-2">
        <span className="text-base">🔍</span>
        <div className="relative flex-1 max-w-[300px]">
          <input
            ref={ref}
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder="Search files... (Ctrl+F)"
            className="w-full bg-[#12101a] border border-[#3c2860] rounded px-3 py-1.5 text-sm text-white placeholder-gray-500 focus:border-cyber-cyan focus:outline-none transition-colors"
          />
          {value && (
            <button
              onClick={() => onChange("")}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
            >
              ✕
            </button>
          )}
        </div>
        {resultCount !== undefined && (
          <span className="text-sm text-gray-400">({resultCount} results)</span>
        )}
      </div>
    );
  }
);
