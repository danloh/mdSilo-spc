import { BrowserRouter, Route, Routes } from "react-router-dom";
import CollaborationPage from "./collaboration/CollaborationPage";
import LandingPage from "./pages/LandingPage";
import SiloPage from "./silo/SiloPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/pad" element={<LandingPage />} />
        <Route path="/pad/app" element={<SiloPage />} />
        <Route path="/pad/:id" element={<CollaborationPage />} />
      </Routes>
    </BrowserRouter>
  );
}
