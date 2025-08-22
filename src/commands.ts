import { invoke } from "@tauri-apps/api/core";
type JsonValue = any;

function tryParseJSON(v: any) {
  if (typeof v !== "string") return v;
  const s = v.trim();
  if (
    (s.startsWith("{") && s.endsWith("}")) ||
    (s.startsWith("[") && s.endsWith("]"))
  ) {
    try {
      return JSON.parse(s);
    } catch (_) {
      return v;
    }
  }
  return v;
}

async function invokeCmd<T = any>(
  cmd: string,
  args?: Record<string, any>
): Promise<T> {
  const res = await invoke(cmd, args ?? {});
  console.log("invokeCmd", { cmd, args, res });
  return tryParseJSON(res) as T;
}

// Exported wrappers
export async function ping(): Promise<string> {
  return invokeCmd("ping");
}

export async function getList(): Promise<JsonValue[]> {
  return invokeCmd("get_list");
}
export async function setList(v: JsonValue[]): Promise<boolean> {
  return invokeCmd("set_list", { v });
}

export async function getContentOfList(): Promise<string> {
  return invokeCmd("get_content_of_list");
}
export async function getItemFromList(id: string): Promise<JsonValue | null> {
  return invokeCmd("get_item_from_list", { id });
}

export async function moveToTrashcan(id: string): Promise<boolean> {
  return invokeCmd("move_to_trashcan", { id });
}
export async function moveManyToTrashcan(ids: string[]): Promise<boolean> {
  return invokeCmd("move_many_to_trashcan", { ids });
}

export async function getTrashcanList(): Promise<JsonValue[]> {
  return invokeCmd("get_trashcan_list");
}
export async function clearTrashcan(): Promise<boolean> {
  return invokeCmd("clear_trashcan");
}
export async function deleteItemFromTrashcan(id: string): Promise<boolean> {
  return invokeCmd("delete_item_from_trashcan", { id });
}
export async function restoreItemFromTrashcan(id: string): Promise<boolean> {
  return invokeCmd("restore_item_from_trashcan", { id });
}

export async function configGet(key: string): Promise<JsonValue | null> {
  return invokeCmd("config_get", { key });
}
export async function configSet(key: string, val: JsonValue): Promise<boolean> {
  return invokeCmd("config_set", { key, val });
}

export async function showItemInFolder(link: string): Promise<boolean> {
  return invokeCmd("show_item_in_folder", { link });
}

export async function getDataDir(): Promise<string> {
  return invokeCmd("get_data_dir");
}
export async function getDefaultDataDir(): Promise<string> {
  return invokeCmd("get_default_data_dir");
}
export async function getBasicData(): Promise<JsonValue> {
  return invokeCmd("get_basic_data");
}

export async function getPathOfSystemHosts(): Promise<string> {
  return invokeCmd("get_path_of_system_hosts");
}
export async function getSystemHosts(): Promise<string> {
  return invokeCmd("get_system_hosts");
}
export async function getHostsContent(id: string): Promise<string> {
  return invokeCmd("get_hosts_content", { id });
}
export async function setHostsContent(
  id: string,
  content: string
): Promise<boolean> {
  return invokeCmd("set_hosts_content", { id, content });
}

export async function setSystemHosts(
  content: string,
  opts?: string | null
): Promise<JsonValue> {
  return invokeCmd("set_system_hosts", { content, opts });
}

export async function getHistoryList(): Promise<JsonValue[]> {
  return invokeCmd("get_history_list");
}
export async function deleteHistory(id: string): Promise<boolean> {
  return invokeCmd("delete_history", { id });
}

export async function refreshHosts(): Promise<JsonValue> {
  return invokeCmd("refresh_hosts");
}

export async function updateTrayTitle(title: string): Promise<boolean> {
  return invokeCmd("update_tray_title", { _title: title });
}

export async function migrateCheck(): Promise<boolean> {
  return invokeCmd("migrate_check");
}
export async function migrateData(): Promise<boolean> {
  return invokeCmd("migrate_data");
}
export async function exportData(): Promise<string | null> {
  return invokeCmd("export_data");
}

export async function cmdGetHistoryList(): Promise<JsonValue[]> {
  return invokeCmd("cmd_get_history_list");
}
export async function cmdDeleteHistory(id: string): Promise<boolean> {
  return invokeCmd("cmd_delete_history", { id });
}
export async function cmdClearHistory(): Promise<boolean> {
  return invokeCmd("cmd_clear_history");
}

export async function tryToRun(): Promise<JsonValue | null> {
  return invokeCmd("try_to_run");
}
export async function checkUpdate(): Promise<string | null> {
  return invokeCmd("check_update");
}

export async function importData(path: string): Promise<boolean> {
  return invokeCmd("import_data", { path });
}
export async function importDataFromUrl(url: string): Promise<boolean> {
  return invokeCmd("import_data_from_url", { url });
}

export async function findGetHistory(): Promise<JsonValue[]> {
  return invokeCmd("find_get_history");
}
export async function findSetHistory(list: JsonValue[]): Promise<boolean> {
  return invokeCmd("find_set_history", { list });
}
export async function findAddHistory(item: JsonValue): Promise<JsonValue[]> {
  return invokeCmd("find_add_history", { item });
}

export async function configAll(): Promise<JsonValue> {
  return invokeCmd("config_all");
}
export async function configUpdate(cfg: string): Promise<boolean> {
  return invokeCmd("config_update", { cfg });
}

export async function closeMainWindow(): Promise<boolean> {
  return invokeCmd("close_main_window");
}
export async function quitApp(): Promise<boolean> {
  return invokeCmd("quit");
}

export async function openUrl(url: string): Promise<boolean> {
  return invokeCmd("open_url", { url });
}

export async function noop(): Promise<string> {
  return invokeCmd("noop");
}

// default export with snake_case aliases for convenience
const commands = {
  ping,
  getList,
  setList,
  getContentOfList,
  getItemFromList,
  moveToTrashcan,
  moveManyToTrashcan,
  getTrashcanList,
  clearTrashcan,
  deleteItemFromTrashcan,
  restoreItemFromTrashcan,
  configGet,
  configSet,
  showItemInFolder,
  getDataDir,
  getDefaultDataDir,
  getBasicData,
  getPathOfSystemHosts,
  getSystemHosts,
  getHostsContent,
  setHostsContent,
  setSystemHosts,
  getHistoryList,
  deleteHistory,
  refreshHosts,
  updateTrayTitle,
  migrateCheck,
  migrateData,
  exportData,
  cmdGetHistoryList,
  cmdDeleteHistory,
  cmdClearHistory,
  tryToRun,
  checkUpdate,
  importData,
  importDataFromUrl,
  findGetHistory,
  findSetHistory,
  findAddHistory,
  configAll,
  configUpdate,
  closeMainWindow,
  quitApp,
  openUrl,
  noop,
};

export default commands;
