import create, { State, StateCreator } from 'zustand';
import createVanilla from 'zustand/vanilla';
import { persist, StateStorage } from 'zustand/middleware';
import produce, { Draft } from 'immer';
import { ArticleType, PodType } from '../reader/types';


export { default as shallowEqual } from 'zustand/shallow';

const immer =
  <T extends State>(
    config: StateCreator<T, (fn: (draft: Draft<T>) => void) => void>
  ): StateCreator<T> =>
  (set, get, api) => config((fn) => set(produce<T>(fn)), get, api);


export type Store = {
  // input end
  currentArticle: ArticleType | null;   // feed article
  setCurrentArticle: Setter<ArticleType | null>;
  currentPod: PodType | null; 
  setCurrentPod: Setter<PodType | null>;
};

type FunctionPropertyNames<T> = {
  // eslint-disable-next-line @typescript-eslint/ban-types
  [K in keyof T]: T[K] extends Function ? K : never;
}[keyof T];

type StoreWithoutFunctions = Omit<Store, FunctionPropertyNames<Store>>;

export type Setter<T> = (value: T | ((value: T) => T)) => void;
export const setter =
  <K extends keyof StoreWithoutFunctions>(
    set: (fn: (draft: Draft<Store>) => void) => void,
    key: K
  ) =>
  (value: Store[K] | ((value: Store[K]) => Store[K])) => {
    if (typeof value === 'function') {
      set((state) => {
        state[key] = value(state[key]);
      });
    } else {
      set((state) => {
        state[key] = value;
      });
    }
  };

export const store = createVanilla<Store>(
  persist(
    immer((set) => ({
      // input end
      currentArticle: null,
      setCurrentArticle: setter(set, 'currentArticle'),
      currentPod: null,
      setCurrentPod: setter(set, 'currentPod'),
    })), 
    {name: 'mdsilo-storage',}
  )
);

export const useStore = create<Store>(store);
