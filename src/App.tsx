import { BrowserRouter, Route, Routes } from "react-router-dom";
import CollaborationPage from "./pages/CollaborationPage";
import LandingPage from "./pages/LandingPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/pad" element={<LandingPage />} />
        <Route path="/pad/:id" element={<CollaborationPage />} />
      </Routes>
    </BrowserRouter>
  );
}
