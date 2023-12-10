import type { GlobalModuleRegistry, GlobalModule } from './types';

export const createModuleRegistry = () => new Proxy(
  Object.create(null) as GlobalModuleRegistry,
  {
    get(mod, id) {
      if (typeof id === 'symbol') {
        console.warn('unable to get module by symbol');
        return;
      }
      return mod[id] ?? (() => {
        throw new Error(`module '${id}' not found`);
      })();
    },
  },
);

export const createModule = (source?: any) => {
  const mod = Object.create(null);
  return (source ? Object.assign(mod, source) : mod) as GlobalModule;
};
