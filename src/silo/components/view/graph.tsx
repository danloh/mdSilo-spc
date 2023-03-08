import { useEffect } from 'react';
import ErrorBoundary from '../misc/ErrorBoundary';
import ForceGraph from '../view/ForceGraph';
import { useStore } from '../../lib/store';

export default function Graph() {
  const isLoaded = useStore((state) => state.isLoaded);
  const setIsLoaded = useStore((state) => state.setIsLoaded);
  const initDir = useStore((state) => state.initDir);
  // console.log("g loaded?", isLoaded);
  useEffect(() => {
    if (!isLoaded && initDir) {
      // loadDir(initDir).then(() => setIsLoaded(true));
    }
  }, [initDir, isLoaded, setIsLoaded]);

  return (
    <ErrorBoundary>
      <ForceGraph className="flex-1" />
    </ErrorBoundary>
  );
}
