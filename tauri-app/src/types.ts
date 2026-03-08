export interface FileItem {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  category: FileCategory;
  usefulness: number;
  modified: number | null;
  child_count: number | null;
  icon: string;
}

export type FileCategory = "MustKeep" | "System" | "Regular" | "Useless" | "Unknown";

export type SortColumn = "Name" | "Size" | "Category" | "Usefulness";
export type SortDirection = "Ascending" | "Descending";

export interface DiskInfo {
  mount_point: string;
  name: string;
  total_space: number;
  available_space: number;
  used_space: number;
  usage_percent: number;
}

export interface BatchDeleteResult {
  deleted: number;
  skipped: number;
  errors: string[];
}
