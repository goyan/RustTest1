use sysinfo::Disks;
use crate::models::DiskInfo;

#[tauri::command]
pub fn list_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut result: Vec<DiskInfo> = disks.list().iter()
        .filter(|d| d.total_space() > 0)
        .map(|d| {
            let total = d.total_space();
            let available = d.available_space();
            let used = total - available;
            let percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            DiskInfo {
                mount_point: d.mount_point().to_string_lossy().to_string(),
                name: d.name().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent: percent,
            }
        })
        .collect();
    result.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    result
}
