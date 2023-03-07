import { useEffect, useState } from "react";
import {
  Box,
  Button,
  Text,
} from "@chakra-ui/react";
import { VscArrowRight } from "react-icons/vsc";
import { useNavigate } from "react-router-dom";
import { gen_id } from "spc-wasm";

const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const idLen = 6;

// can set expire via `/?expire={num}`
function getExpire() {
  let search = window.location.search;
  if (search.trim()) {
    let split0 = search.split("?expire=")[1] || "";
    let split1 = split0.split("&")[0] || "";
    let num = Number(split1);
    if (Number.isInteger(num)) {
      return num
    }
  }
  return 3600 * 24;
}

function genHashId() {
  let id = "";
  for (let i = 0; i < idLen; i++) {
    id += chars[Math.floor(Math.random() * chars.length)];
  }
  return gen_id(BigInt(getExpire()), id);
}

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
