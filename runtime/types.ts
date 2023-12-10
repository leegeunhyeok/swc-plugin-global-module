export type GlobalModuleId = string;
export type GlobalModuleRegistry = Record<GlobalModuleId, GlobalModule>;
export type GlobalModule<T = unknown> = Record<string, T>;

export interface GlobalModuleApi {
  registry: GlobalModuleRegistry;
  esm: (
    id: GlobalModuleId,
    exportedModule: GlobalModule,
    ...reExportedModules: GlobalModule[]
  ) => void;
  helpers: {
    asWildcard: (targetModule: GlobalModule) => GlobalModule;
  }
}
