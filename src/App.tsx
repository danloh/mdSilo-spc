import { Suspense, lazy } from 'react';
import { BrowserRouter, Route, Routes } from "react-router-dom";

const PadPage = lazy(() => import('./mdpad/PadPage'));
const EditorPage = lazy(() => import('./mdpad/EditorPage'));
const ReaderPage = lazy(() => import('./reader/ReaderPage'));
const NotePage = lazy(() => import('./writer/NotePage'));
const NoteEditor = lazy(() => import('./writer/NoteEditor'));
const LandingPage = lazy(() => import('./pages/LandingPage'));

function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={<div>Loading...</div>}>
        <Routes>
          <Route path="/app" element={<LandingPage />} />
          <Route path="/app/reader" element={<ReaderPage />} />
          <Route path="/app/editor" element={<EditorPage />} />
          <Route path="/app/notes" element={<NotePage />} />
          <Route path="/app/write/:id" element={<NoteEditor />} />
          <Route path="/app/pad" element={<PadPage />} />
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}

export default App;
