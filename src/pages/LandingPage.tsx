import { useEffect, useState } from "react";
import {
  Box,
  Button,
  Text,
} from "@chakra-ui/react";
import { VscArrowRight } from "react-icons/vsc";
import { useNavigate } from "react-router-dom";
import { genHashId } from "../useHash";


export default function LandingPage() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [showButton, setShowButton] = useState(false);

  useEffect(() => {
    setTimeout(() => setShowButton(true), 500);
  }, []);

  function handleClick() {
    setLoading(true);
    // Arbitrary delay for suspense reasons.
    setTimeout(() => {
      const id = genHashId();
      navigate(`/pad/${id}`);
    }, 500);
  }

  return (
    
    <Button
      size="lg"
      colorScheme="blue"
      fontSize="2xl"
      textTransform="uppercase"
      h={14}
      rightIcon={<VscArrowRight />}
      onClick={handleClick}
      isLoading={loading}
      opacity={showButton ? 1 : 0}
    >
      Start creating
    </Button>
          
  );
}
