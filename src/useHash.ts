import { useEffect, useState } from "react";
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

export function genHash() {
  let id = "";
  for (let i = 0; i < idLen; i++) {
    id += chars[Math.floor(Math.random() * chars.length)];
  }
  let hashid = gen_id(BigInt(getExpire()), id);
  return hashid;
}

function getHash() {
  if (!window.location.hash) {
    let hashid = genHash();
    window.history.replaceState(null, "", "#" + hashid);
  }
  return window.location.hash.slice(1);
}

export default function useHash() {
  const [hash, setHash] = useState(getHash);

  useEffect(() => {
    const handler = () => setHash(getHash());
    window.addEventListener("hashchange", handler);
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  return hash;
}
