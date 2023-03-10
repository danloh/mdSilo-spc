import { BrowserRouter, Route, Routes } from "react-router-dom";
import EditorPage from "./mdpad/PadPage";
import LandingPage from "./pages/LandingPage";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/app" element={<LandingPage />} />
        <Route path="/app/pad" element={<EditorPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
